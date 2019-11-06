use anyhow::Result;
use gfx::Device;
use log::info;
use nalgebra::Vector3;

use voxel_rs_common::{
    block::Block,
    network::{messages::ToClient, messages::ToServer, Client, ClientEvent},
    registry::Registry,
    world::{chunk::CHUNK_SIZE, World},
};

use crate::world::camera::Camera;
use crate::{
    fps::FpsCounter,
    input::InputState,
    mesh::Mesh,
    physics::aabb::AABB,
    settings::Settings,
    ui::{renderer::UiRenderer, Ui},
    window::{Gfx, State, StateTransition, WindowData, WindowFlags},
    world::{
        meshing::{greedy_meshing as meshing, AdjChunkOccl},
        renderer::WorldRenderer,
    },
};
use std::collections::HashSet;
use std::time::Instant;

/// State of a singleplayer world
pub struct SinglePlayer {
    fps_counter: FpsCounter,
    ui: Ui,
    ui_renderer: UiRenderer,
    world: World,
    world_renderer: WorldRenderer,
    camera: Camera,
    player: AABB,
    #[allow(dead_code)] // TODO: remove this
    block_registry: Registry<Block>,
    client: Box<dyn Client>,
}

impl SinglePlayer {
    pub fn new_factory(client: Box<dyn Client>) -> crate::window::StateFactory {
        Box::new(move |settings, gfx| Self::new(settings, gfx, client))
    }

    pub fn new(
        _settings: &mut Settings,
        gfx: &mut Gfx,
        mut client: Box<dyn Client>,
    ) -> Result<Box<dyn State>> {
        info!("Launching singleplayer");
        // Wait for data from the server
        let data = loop {
            match client.receive_event() {
                ClientEvent::ServerMessage(ToClient::GameData(data)) => break data,
                _ => (),
            }
        };
        info!("Received game data from the server");
        // Load texture atlas
        let texture_atlas = crate::texture::load_image(&mut gfx.factory, data.texture_atlas)?;

        let world_renderer = WorldRenderer::new(gfx, data.meshes, texture_atlas);

        Ok(Box::new(Self {
            fps_counter: FpsCounter::new(),
            ui: Ui::new(),
            ui_renderer: UiRenderer::new(gfx)?,
            world: World::new(),
            world_renderer: world_renderer?,
            camera: {
                let mut cam = Camera::new();
                cam.position = Vector3::new(0.4, 1.6, 0.4);
                cam
            },
            player: AABB::new(Vector3::new(0.0, 0.0, 0.0), (0.8, 1.8, 0.8)),
            block_registry: data.blocks,
            client,
        }))
    }
}

impl State for SinglePlayer {
    fn update(
        &mut self,
        _settings: &mut Settings,
        keyboard_state: &InputState,
        _data: &WindowData,
        flags: &mut WindowFlags,
        seconds_delta: f64,
        gfx: &mut Gfx,
    ) -> Result<StateTransition> {
        let mut chunk_updates = HashSet::new();
        // Handle server messages
        loop {
            match self.client.receive_event() {
                ClientEvent::NoEvent => break,
                ClientEvent::ServerMessage(message) => match message {
                    ToClient::Chunk(chunk) => {
                        // TODO: make sure this only happens once
                        self.world.set_chunk(chunk.to_chunk());
                        // Queue chunks for meshing
                        for i in -1..=1 {
                            for j in -1..=1 {
                                for k in -1..=1 {
                                    chunk_updates.insert(chunk.pos.offset(i, j, k));
                                }
                            }
                        }
                    }
                    ToClient::GameData(_) => {}
                },
                ClientEvent::Disconnected => unimplemented!("server disconnected"),
                ClientEvent::Connected => {}
            }
        }

        if self.ui.should_update_camera() {
            let delta_move = self.camera.get_movement(seconds_delta, keyboard_state);
            let delta_move = self.player.move_check_collision(&self.world, delta_move);

            self.camera.position += delta_move;

            // TODO: real physics handling
            let p = self.camera.position;
            self.client.send(ToServer::SetPos((p[0], p[1], p[2])));
        }

        // Update meshing
        let mut quad_buffer = Vec::new();
        for chunk_pos in chunk_updates.into_iter() {
            if let Some(chunk) = self.world.get_chunk(chunk_pos) {
                let t1 = Instant::now();
                let (vertices, indices, _, _) = meshing(
                    chunk,
                    Some(AdjChunkOccl::create_from_world(&self.world, chunk_pos, &self.world_renderer.block_meshes)),
                    &self.world_renderer.block_meshes,
                    &mut quad_buffer,
                );
                let t2 = Instant::now();
                let pos = (
                    (chunk.pos.px * CHUNK_SIZE as i64) as f32,
                    (chunk.pos.py * CHUNK_SIZE as i64) as f32,
                    (chunk.pos.pz * CHUNK_SIZE as i64) as f32,
                );
                // TODO: reuse existing meshes when possible if that bottlenecks
                let chunk_mesh = Mesh::new(pos, vertices, indices, &mut gfx.factory);
                let t3 = Instant::now();
                info!("Meshing took {} ms\nUpdating the mesh took {} ms", (t2 - t1).as_millis(), (t3 - t2).as_millis());
                self.world_renderer.update_chunk_mesh(chunk.pos, chunk_mesh);
            }
        }

        // TODO: drop chunks that are too far away

        flags.hide_and_center_cursor = self.ui.should_capture_mouse();

        if self.ui.should_exit() {
            //Ok(StateTransition::ReplaceCurrent(Box::new(crate::mainmenu::MainMenu::new)))
            Ok(StateTransition::CloseWindow)
        } else {
            Ok(StateTransition::KeepCurrent)
        }
    }

    fn render(
        &mut self,
        _settings: &Settings,
        gfx: &mut Gfx,
        data: &WindowData,
    ) -> Result<StateTransition> {
        // Count fps
        self.fps_counter.add_frame();

        // Clear buffers
        gfx.encoder
            .clear(&gfx.color_buffer, crate::window::CLEAR_COLOR);
        gfx.encoder
            .clear_depth(&gfx.depth_buffer, crate::window::CLEAR_DEPTH);
        // Draw world
        self.world_renderer.render(gfx, data, &self.camera)?;
        // Clear depth
        gfx.encoder
            .clear_depth(&gfx.depth_buffer, crate::window::CLEAR_DEPTH);
        // Draw ui
        self.ui
            .rebuild(&self.camera, self.fps_counter.fps(), data)?;
        self.ui_renderer.render(gfx, &data, &self.ui.ui)?;
        // Flush and swap buffers
        gfx.encoder.flush(&mut gfx.device);
        gfx.context.swap_buffers()?;
        gfx.device.cleanup();

        Ok(StateTransition::KeepCurrent)
    }

    fn handle_mouse_motion(&mut self, _settings: &Settings, delta: (f64, f64)) {
        if self.ui.should_update_camera() {
            self.camera.update_cursor(delta.0, delta.1);
        }
    }

    fn handle_cursor_movement(&mut self, logical_position: glutin::dpi::LogicalPosition) {
        self.ui.cursor_moved(logical_position);
    }

    fn handle_mouse_state_changes(
        &mut self,
        changes: Vec<(glutin::MouseButton, glutin::ElementState)>,
    ) {
        self.ui.handle_mouse_state_changes(changes);
    }

    fn handle_key_state_changes(&mut self, changes: Vec<(u32, glutin::ElementState)>) {
        self.ui.handle_key_state_changes(changes);
    }
}
