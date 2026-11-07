use wayland_client::{protocol::{ wl_registry, wl_compositor, wl_surface }, Connection, Dispatch, QueueHandle, EventQueue};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1;

struct App {
    compositor_name: Option<u32>,
    layer_shell_name: Option<u32>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for App {
    fn event(
        state: &mut Self,
        _:&wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    ) {
        match event {
            wl_registry::Event::Global { name, interface, .. } => {
                if interface == "wl_compositor" {
                    state.compositor_name = Some(name);
                }

                if interface == "zwlr_layer_shell_v1" {
                    state.layer_shell_name = Some(name);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for App {
    fn event(
        _state: &mut Self,
        _: &wl_compositor::WlCompositor,
        _event: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    )
    {}
}

impl Dispatch<wl_surface::WlSurface, ()> for App {
    fn event(
        _state: &mut Self,
        _: &wl_surface::WlSurface,
        _event: wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    )
    {}
}

impl Dispatch<zwlr_layer_shell_v1::ZwlrLayerShellV1, ()> for App {
    fn event(
        _state: &mut Self,
        _: &zwlr_layer_shell_v1::ZwlrLayerShellV1,
        _event: zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    )
    {}
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue: EventQueue<App> = conn.new_event_queue();
    let qh = event_queue.handle();

    let registry = display.get_registry(&qh, ());
    let mut state = App{compositor_name: None, layer_shell_name: None};

    event_queue.roundtrip(&mut state).unwrap();

    let compositor_name = state.compositor_name.expect("wl_compositor not found");
    let compositor: wl_compositor::WlCompositor = registry.bind(compositor_name, 1, &qh, ());

    let surface: wl_surface::WlSurface = compositor.create_surface(&qh, ());

    let layer_shell_name = state.layer_shell_name.expect("zwlr_layer_shell_v1 not found");
    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = registry.bind(layer_shell_name, 1, &qh, ());

    println!("comp_name: {compositor_name}");
    println!("zwlr_name: {layer_shell_name}");

}
