use std::time::Duration;

use macroquad::prelude::*;
use miniphys::cloth::Cloth;
use nalgebra::base::Vector2;

#[macroquad::main("Cloth Simulation")]
async fn main() {
    // Initialize the cloth simulation
    let mut cloth = Cloth::new(30, 20, 40.0);

    // Variables for interaction
    let mut dragging = false;
    let mut right_button = false;

    loop {
        // Simulation step

        cloth.simulate(Duration::from_secs_f32(get_frame_time()));

        // Handle input
        if is_mouse_button_pressed(MouseButton::Right) {
            // Start dragging particles
            dragging = true;
            right_button = true;
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_pos = Vector2::new(mouse_x.into(), mouse_y.into());
            cloth.select_particles(mouse_pos, 30.0);
        }

        if is_mouse_button_released(MouseButton::Right) && right_button {
            // Stop dragging particles
            dragging = false;
            right_button = false;
            cloth.clear_selection();
        }

        if is_mouse_button_pressed(MouseButton::Left) {
            // Cut constraints at mouse position
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_pos = Vector2::new(mouse_x.into(), mouse_y.into());
            cloth.cut_at_mouse(mouse_pos);
        }

        if dragging && right_button {
            let (mouse_x, mouse_y) = mouse_position();
            let mouse_pos = Vector2::new(mouse_x.into(), mouse_y.into());
            cloth.move_selected_particles(mouse_pos);
        }

        // Clear the screen
        clear_background(BLACK);

        // Draw constraints
        for constraint in cloth.constraints() {
            let (index_a, index_b) = constraint.particles();
            let p1 = cloth.particles()[index_a].position();
            let p2 = cloth.particles()[index_b].position();

            draw_line(
                p1.x as f32,
                p1.y as f32,
                p2.x as f32,
                p2.y as f32,
                1.0,
                WHITE,
            );
        }

        // Draw particles
        for particle in cloth.particles() {
            let pos = particle.position();
            draw_circle(pos.x as f32, pos.y as f32, 3.0, YELLOW);
        }

        // Highlight selected particles
        for &index in cloth.selected_particles() {
            let pos = cloth.particles()[index].position();
            draw_circle_lines(pos.x as f32, pos.y as f32, 5.0, 2.0, RED);
        }

        // Draw FPS
        draw_text(&format!("FPS: {}", get_fps()), 20.0, 20.0, 20.0, GREEN);

        // End drawing and wait for the next frame
        next_frame().await;
    }
}
