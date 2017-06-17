extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use opengl_graphics::{ GlGraphics, OpenGL };
use glutin_window::GlutinWindow as Window;

use std::thread;
use std::time;
use std::sync::mpsc;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    color: [f32; 4],
    background: [f32; 4],
    rotation: f64,   // Rotation for the square.
    position: (f64, f64),
    velocity: (f64, f64),
    frame_count: i32,

    color_sender: mpsc::Sender<([f32; 4], [f32; 4])>,
    color_receiver: mpsc::Receiver<([f32; 4], [f32; 4])>
}

impl App {
    fn move_up(&mut self) {
        self.velocity.1 = -1.0;
    }
    fn move_down(&mut self) {
        self.velocity.1 = 1.0;
    }
    fn move_left(&mut self) {
        self.velocity.0 = -1.0;
    }
    fn move_right(&mut self) {
        self.velocity.0 = 1.0;
    }
    fn stop_vertical(&mut self) {
        self.velocity.1 = 0.0;
    }
    fn stop_horizontal(&mut self) {
        self.velocity.0 = 0.0;
    }
    
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;
        
        let square = rectangle::square(0.0, 0.0, 50.0);
        let rotation = self.rotation;
        let (x, y) = self.position;

        let color = self.color;
        let background = self.background;
        
        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(background, gl);

            let transform = c.transform.trans(x, y)
                                       .rot_rad(rotation)
                                       .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            rectangle(color, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.frame_count += 1;
        // Rotate 2 radians per second.
        self.rotation += 2.0 * args.dt;
        self.position.0 += self.velocity.0;
        self.position.1 += self.velocity.1;
        
        if self.frame_count % 300 == 0 {
            do_expensive_work(self.color, self.color_sender.clone());
        }
        
        if let Ok((color, background)) = self.color_receiver.try_recv() {
            self.color = color;
            self.background = background;
        }
    }

}

fn do_expensive_work(current_color: [f32; 4], color_sender: mpsc::Sender<([f32; 4], [f32; 4])>) {
    thread::spawn(move || {
        //rather than doing work immediately, let's take a nap
        thread::sleep(time::Duration::from_secs(2));
        let b = current_color[2] + 1.0;
        let g = current_color[1] + if b > 1.0 { 1.0 } else { 0.0 };
        let r = current_color[0] + if g > 1.0 { 1.0 } else { 0.0 };
        let send_result = color_sender.send(([r%2.0, g%2.0, b%2.0, 1.0], [(r+1.0)%2.0, (g+1.0)%2.0, (b+1.0)%2.0, 1.0]));

        if let Err(send_err) = send_result {
            println!("Error on sending colors back to main thread: {}", send_err);
        }
    });
}


fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "spinning-square",
            [400, 400]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let (color_sender, color_receiver) = mpsc::channel();
    let mut app = App {
        gl: GlGraphics::new(opengl),
        color: [0.0, 0.0, 0.0, 1.0],
        background: [1.0, 1.0, 1.0, 1.0],
        rotation: 0.0,
        position: (200.0, 200.0),
        velocity: (0.0, 0.0),
        frame_count: 0,
        color_sender: color_sender,
        color_receiver: color_receiver
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Up => app.move_up(),
                Key::Down => app.move_down(),
                Key::Left => app.move_left(),
                Key::Right => app.move_right(),
                _ => {}
            }
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            match key {
                Key::Up | Key::Down => app.stop_vertical(),
                Key::Left | Key::Right => app.stop_horizontal(),
                _ => {}
            }
        }
    }
}
