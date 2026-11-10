use wayland_client::{protocol::{wl_registry, wl_compositor, wl_surface, wl_shm, wl_shm_pool, wl_buffer}, Connection, Dispatch, QueueHandle, EventQueue};
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;
use rustix::fs::{memfd_create, MemfdFlags, ftruncate};
use rustix::mm::{mmap, MapFlags, ProtFlags};
use std::os::fd::AsFd;

struct App {
    compositor_name: Option<u32>,
    layer_shell_name: Option<u32>,
    shm_name: Option<u32>,
    width: u32,
    height: u32,
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

                if interface == "wl_shm" {
                    state.shm_name = Some(name)
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
        _: &QueueHandle<App>,
    )
    {}
}

impl Dispatch<zwlr_layer_surface_v1::ZwlrLayerSurfaceV1, ()> for App {
    fn event(
        state: &mut Self,
        proxy: &zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    )
    {
        match event {
            zwlr_layer_surface_v1::Event::Configure { serial, width, height } => {
                println!("Configure: {width}x{height}, serial={serial}");
                proxy.ack_configure(serial);
                state.width = width;
                state.height = height;
            }

            _ => {}
        }
    }
}

impl Dispatch<wl_shm::WlShm, ()> for App {
    fn event(
        _state: &mut Self,
        _: &wl_shm::WlShm,
        _event: wl_shm::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    )
    {}
}

impl Dispatch<wl_shm_pool::WlShmPool, ()> for App {
    fn event(
        _state: &mut Self,
        _: &wl_shm_pool::WlShmPool,
        _event: wl_shm_pool::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    )
    {}
}

impl Dispatch<wl_buffer::WlBuffer, ()> for App {
    fn event(
        _state: &mut Self,
        _: &wl_buffer::WlBuffer,
        _event: wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<App>,
    )
    {}
}

fn main() {
    let conn = Connection::connect_to_env().unwrap();
    let display = conn.display();

    let mut event_queue: EventQueue<App> = conn.new_event_queue();
    let qh = event_queue.handle();

    let registry = display.get_registry(&qh, ());
    let mut state = App{compositor_name: None, layer_shell_name: None, shm_name: None, width: 0, height: 0};

    event_queue.roundtrip(&mut state).unwrap();

    let compositor_name = state.compositor_name.expect("wl_compositor not found");
    let layer_shell_name = state.layer_shell_name.expect("zwlr_layer_shell_v1 not found");
    let shm_name = state.shm_name.expect("wl_shm not found");

    let layer_shell: zwlr_layer_shell_v1::ZwlrLayerShellV1 = registry.bind(layer_shell_name, 1, &qh, ());
    let compositor: wl_compositor::WlCompositor = registry.bind(compositor_name, 1, &qh, ());
    let shm: wl_shm::WlShm = registry.bind(shm_name, 1, &qh, ());
    
    let surface: wl_surface::WlSurface = compositor.create_surface(&qh, ());
    let layer_surface: zwlr_layer_surface_v1::ZwlrLayerSurfaceV1 = layer_shell.get_layer_surface(&surface, None, Layer::Top, "oxalis".to_string(), &qh, ());

    layer_surface.set_size(0, 19);
    layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    layer_surface.set_exclusive_zone(19);
    
    surface.commit();

    event_queue.roundtrip(&mut state).unwrap();

    let stride = state.width * 4;
    let size = stride * state.height;

    let fd = memfd_create("oxalis-buffer", MemfdFlags::CLOEXEC).unwrap();
    ftruncate(&fd, u64::from(size)).unwrap();
    
    let pool: wl_shm_pool::WlShmPool = shm.create_pool (
        fd.as_fd(),
        i32::try_from(size).unwrap(),
        &qh,
        ()
    );
    
    let buffer: wl_buffer::WlBuffer = pool.create_buffer (
        0,
        i32::try_from(state.width).unwrap(),
        i32::try_from(state.height).unwrap(),
        i32::try_from(stride).unwrap(),
        wl_shm::Format::Xrgb8888,
        &qh,
        ()
    );

    let ptr = unsafe {
        mmap (
            std::ptr::null_mut(),
            size as usize,
            ProtFlags::READ | ProtFlags::WRITE,
            MapFlags::SHARED,
            fd.as_fd(),
            0
        )
        .unwrap()
    };

    let pixels = unsafe {
        std::slice::from_raw_parts_mut(ptr.cast::<u8>(), size as usize)
    };

    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0x35, 0x55, 0x35, 0x00]);
    }

    surface.attach(Some(&buffer), 0, 0);
    surface.commit();

    loop {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }
}
