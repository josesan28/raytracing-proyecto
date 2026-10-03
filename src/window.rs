#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::c_void;
    use std::io;
    use std::mem::size_of;
    use std::ptr::{null, null_mut};
    use std::time::{Duration, Instant};

    use crate::camera::Camera;
    use crate::color::Color;
    use crate::framebuffer::Framebuffer;
    use crate::math::Vec3;
    use crate::output::write_png;
    use crate::renderer::{ECONOMY_OPTIONS, RenderOptions, render_refined};
    use crate::scene::Scene;
    use crate::window_renderer::{RenderMode, RenderRequest, WindowRenderer};

    type Handle = *mut c_void;
    type WindowHandle = Handle;
    type DeviceContext = Handle;
    type InstanceHandle = Handle;

    const WINDOW_WIDTH: usize = 1280;
    const WINDOW_HEIGHT: usize = 720;
    const REFINE_DELAY: Duration = Duration::from_millis(200);
    const SAVED_RENDER_WIDTH: usize = 1600;
    const SAVED_RENDER_HEIGHT: usize = 900;
    const MOUSE_ORBIT_SENSITIVITY: f32 = 0.0035;
    const KEYBOARD_ORBIT_STEP: f32 = 0.055;
    const ZOOM_STEP: f32 = 1.0;

    const WM_DESTROY: u32 = 0x0002;
    const WM_SIZE: u32 = 0x0005;
    const WM_PAINT: u32 = 0x000F;
    const WM_CLOSE: u32 = 0x0010;
    const WM_QUIT: u32 = 0x0012;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_MOUSEMOVE: u32 = 0x0200;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_MOUSEWHEEL: u32 = 0x020A;
    const WM_CAPTURECHANGED: u32 = 0x0215;

    const VK_ESCAPE: usize = 0x1B;
    const VK_LEFT: usize = 0x25;
    const VK_UP: usize = 0x26;
    const VK_RIGHT: usize = 0x27;
    const VK_DOWN: usize = 0x28;
    const VK_ADD: usize = 0x6B;
    const VK_SUBTRACT: usize = 0x6D;
    const VK_OEM_PLUS: usize = 0xBB;
    const VK_OEM_MINUS: usize = 0xBD;

    const CS_OWNDC: u32 = 0x0020;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
    const WS_VISIBLE: u32 = 0x10000000;
    const CW_USEDEFAULT: i32 = i32::MIN;
    const PM_REMOVE: u32 = 0x0001;
    const SW_SHOW: i32 = 5;
    const DIB_RGB_COLORS: u32 = 0;
    const SRCCOPY: u32 = 0x00CC0020;
    const BI_RGB: u32 = 0;
    const COLORONCOLOR: i32 = 3;
    const IDC_ARROW: usize = 32512;

    #[repr(C)]
    struct WindowClass {
        style: u32,
        window_procedure:
            Option<unsafe extern "system" fn(WindowHandle, u32, usize, isize) -> isize>,
        class_extra: i32,
        window_extra: i32,
        instance: InstanceHandle,
        icon: Handle,
        cursor: Handle,
        background: Handle,
        menu_name: *const u16,
        class_name: *const u16,
    }

    #[repr(C)]
    #[derive(Clone, Copy, Default)]
    struct Point {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Message {
        window: WindowHandle,
        message: u32,
        word_parameter: usize,
        long_parameter: isize,
        time: u32,
        point: Point,
        private: u32,
    }

    impl Default for Message {
        fn default() -> Self {
            Self {
                window: null_mut(),
                message: 0,
                word_parameter: 0,
                long_parameter: 0,
                time: 0,
                point: Point::default(),
                private: 0,
            }
        }
    }

    #[repr(C)]
    #[derive(Default)]
    struct Rectangle {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[repr(C)]
    struct PaintData {
        context: DeviceContext,
        erase_background: i32,
        paint_rectangle: Rectangle,
        restore: i32,
        incremental_update: i32,
        reserved: [u8; 32],
    }

    impl Default for PaintData {
        fn default() -> Self {
            Self {
                context: null_mut(),
                erase_background: 0,
                paint_rectangle: Rectangle::default(),
                restore: 0,
                incremental_update: 0,
                reserved: [0; 32],
            }
        }
    }

    #[repr(C)]
    struct BitmapInfoHeader {
        size: u32,
        width: i32,
        height: i32,
        planes: u16,
        bit_count: u16,
        compression: u32,
        image_size: u32,
        x_pixels_per_meter: i32,
        y_pixels_per_meter: i32,
        colors_used: u32,
        important_colors: u32,
    }

    #[repr(C)]
    struct BitmapInfo {
        header: BitmapInfoHeader,
        colors: [u32; 1],
    }

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SetProcessDPIAware() -> i32;
        fn SetWindowTextW(window: WindowHandle, text: *const u16) -> i32;
        fn RegisterClassW(window_class: *const WindowClass) -> u16;
        fn AdjustWindowRect(rectangle: *mut Rectangle, style: u32, has_menu: i32) -> i32;
        fn CreateWindowExW(
            extended_style: u32,
            class_name: *const u16,
            window_name: *const u16,
            style: u32,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
            parent: WindowHandle,
            menu: Handle,
            instance: InstanceHandle,
            parameter: *mut c_void,
        ) -> WindowHandle;
        fn DefWindowProcW(
            window: WindowHandle,
            message: u32,
            word_parameter: usize,
            long_parameter: isize,
        ) -> isize;
        fn DestroyWindow(window: WindowHandle) -> i32;
        fn PostQuitMessage(exit_code: i32);
        fn PeekMessageW(
            message: *mut Message,
            window: WindowHandle,
            minimum: u32,
            maximum: u32,
            remove: u32,
        ) -> i32;
        fn TranslateMessage(message: *const Message) -> i32;
        fn DispatchMessageW(message: *const Message) -> isize;
        fn ShowWindow(window: WindowHandle, command: i32) -> i32;
        fn UpdateWindow(window: WindowHandle) -> i32;
        fn GetClientRect(window: WindowHandle, rectangle: *mut Rectangle) -> i32;
        fn ValidateRect(window: WindowHandle, rectangle: *const Rectangle) -> i32;
        fn BeginPaint(window: WindowHandle, paint_data: *mut PaintData) -> DeviceContext;
        fn EndPaint(window: WindowHandle, paint_data: *const PaintData) -> i32;
        fn GetDC(window: WindowHandle) -> DeviceContext;
        fn ReleaseDC(window: WindowHandle, context: DeviceContext) -> i32;
        fn LoadCursorW(instance: InstanceHandle, cursor_name: *const u16) -> Handle;
        fn SetCapture(window: WindowHandle) -> WindowHandle;
        fn ReleaseCapture() -> i32;
        fn WaitMessage() -> i32;
        fn MsgWaitForMultipleObjects(
            count: u32,
            handles: *const Handle,
            wait_all: i32,
            milliseconds: u32,
            wake_mask: u32,
        ) -> u32;
    }

    #[link(name = "gdi32")]
    unsafe extern "system" {
        fn SetStretchBltMode(context: DeviceContext, stretch_mode: i32) -> i32;
        fn StretchDIBits(
            context: DeviceContext,
            destination_x: i32,
            destination_y: i32,
            destination_width: i32,
            destination_height: i32,
            source_x: i32,
            source_y: i32,
            source_width: i32,
            source_height: i32,
            bits: *const c_void,
            bitmap_info: *const BitmapInfo,
            usage: u32,
            raster_operation: u32,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetModuleHandleW(module_name: *const u16) -> InstanceHandle;
    }

    unsafe extern "system" fn window_procedure(
        window: WindowHandle,
        message: u32,
        word_parameter: usize,
        long_parameter: isize,
    ) -> isize {
        match message {
            WM_PAINT => {
                let mut paint_data = PaintData::default();
                unsafe {
                    BeginPaint(window, &mut paint_data);
                    EndPaint(window, &paint_data);
                }
                0
            }
            WM_CLOSE => {
                unsafe { DestroyWindow(window) };
                0
            }
            WM_DESTROY => {
                unsafe { PostQuitMessage(0) };
                0
            }
            _ => unsafe { DefWindowProcW(window, message, word_parameter, long_parameter) },
        }
    }

    pub fn run(scene: &Scene) -> io::Result<()> {
        unsafe {
            SetProcessDPIAware();
        }

        let class_name = wide_string("CpuRaytracerWindow");
        let title = wide_string(
            "Ruinas del templo | Arrastra: rotar | Rueda: zoom | WASD: rotar | P: guardar",
        );
        let instance = unsafe { GetModuleHandleW(null()) };
        let window_class = WindowClass {
            style: CS_OWNDC,
            window_procedure: Some(window_procedure),
            class_extra: 0,
            window_extra: 0,
            instance,
            icon: null_mut(),
            cursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW as *const u16) },
            background: null_mut(),
            menu_name: null(),
            class_name: class_name.as_ptr(),
        };

        if unsafe { RegisterClassW(&window_class) } == 0 {
            return Err(io::Error::last_os_error());
        }

        let window_style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
        let mut window_rectangle = Rectangle {
            left: 0,
            top: 0,
            right: WINDOW_WIDTH as i32,
            bottom: WINDOW_HEIGHT as i32,
        };
        if unsafe { AdjustWindowRect(&mut window_rectangle, window_style, 0) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let window_width = window_rectangle.right - window_rectangle.left;
        let window_height = window_rectangle.bottom - window_rectangle.top;

        let window = unsafe {
            CreateWindowExW(
                0,
                class_name.as_ptr(),
                title.as_ptr(),
                window_style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window_width,
                window_height,
                null_mut(),
                null_mut(),
                instance,
                null_mut(),
            )
        };

        if window.is_null() {
            return Err(io::Error::last_os_error());
        }

        unsafe {
            ShowWindow(window, SW_SHOW);
            UpdateWindow(window);
        }

        let renderer = WindowRenderer::default();
        std::thread::scope(|scope| {
            scope.spawn(|| renderer.run(scene));
            let result = event_loop(window, scene, &renderer);
            // También despierta y termina el trabajador si la ventana devuelve un error.
            renderer.stop();
            result
        })
    }

    fn event_loop(
        window: WindowHandle,
        scene: &Scene,
        renderer: &WindowRenderer,
    ) -> io::Result<()> {
        let mut camera = initial_camera();
        let mut mode = RenderMode::Adaptive;
        let mut economy = ECONOMY_OPTIONS;
        let mut generation = 0_u64;
        let mut needs_refine = false;
        let mut last_input = Instant::now();
        show_mode(window, mode, economy);
        println!("F1: adaptativo | F2: calidad constante | F3: económico");
        println!(
            "En económico: F4 sombras, F5 reflejos, F6 refracción, F7 mapas normales, F8 antialiasing."
        );
        println!(
            "Arrastra o WASD: rotar | Rueda o +/-: zoom | R: reiniciar cámara | P: guardar PNG con calidad máxima."
        );
        let mut display_pixels = vec![0_u32; WINDOW_WIDTH * WINDOW_HEIGHT];
        let mut display_size = (WINDOW_WIDTH, WINDOW_HEIGHT);
        let mut requested_size = (WINDOW_WIDTH, WINDOW_HEIGHT);
        let mut requested_version = 0_u64;
        let mut render_pending = false;
        let mut needs_render = true;
        let mut needs_present = true;
        let mut running = true;
        let mut dragging = false;
        let mut previous_mouse = (0_i32, 0_i32);

        while running {
            let mut message = Message::default();
            while unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if message.message == WM_QUIT {
                    running = false;
                    break;
                }

                match message.message {
                    WM_SIZE => needs_render = true,
                    WM_PAINT => needs_present = true,
                    WM_KEYDOWN => {
                        // El bit 30 distingue repetición automática: cada pulsación
                        // cambia una opción una sola vez aunque se mantenga la tecla.
                        let repeated = message.long_parameter & (1 << 30) != 0;
                        if !repeated
                            && handle_mode_key(message.word_parameter, &mut mode, &mut economy)
                        {
                            generation += 1;
                            needs_render = true;
                            show_mode(window, mode, economy);
                        } else if handle_key(message.word_parameter, &mut camera, scene)? {
                            needs_render = true;
                        }
                        if message.word_parameter == VK_ESCAPE {
                            unsafe { DestroyWindow(window) };
                        }
                    }
                    WM_LBUTTONDOWN => {
                        dragging = true;
                        previous_mouse = mouse_position(message.long_parameter);
                        unsafe { SetCapture(window) };
                    }
                    WM_LBUTTONUP => {
                        dragging = false;
                        unsafe { ReleaseCapture() };
                    }
                    WM_CAPTURECHANGED => dragging = false,
                    WM_MOUSEMOVE if dragging => {
                        let current_mouse = mouse_position(message.long_parameter);
                        let delta_x = current_mouse.0 - previous_mouse.0;
                        let delta_y = current_mouse.1 - previous_mouse.1;
                        if delta_x != 0 || delta_y != 0 {
                            camera.orbit(
                                -(delta_x as f32) * MOUSE_ORBIT_SENSITIVITY,
                                -(delta_y as f32) * MOUSE_ORBIT_SENSITIVITY,
                            );
                            previous_mouse = current_mouse;
                            needs_render = true;
                        }
                    }
                    WM_MOUSEWHEEL => {
                        let wheel_delta = ((message.word_parameter >> 16) as u16 as i16) as f32;
                        camera.zoom_proportional(wheel_delta / 120.0 * ZOOM_STEP);
                        needs_render = true;
                    }
                    _ => {}
                }

                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }

            if !running {
                break;
            }

            let mut client = Rectangle::default();
            if unsafe { GetClientRect(window, &mut client) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let width = (client.right - client.left).max(0) as usize;
            let height = (client.bottom - client.top).max(0) as usize;
            // Una ventana minimizada no solicita nuevos cuadros.
            if width == 0 || height == 0 {
                unsafe { WaitMessage() };
                continue;
            }
            if (width, height) != requested_size {
                needs_render = true;
            }

            if needs_render {
                requested_version += 1;
                requested_size = (width, height);
                last_input = Instant::now();
                needs_refine = mode == RenderMode::Adaptive;
                renderer.request(RenderRequest {
                    version: requested_version,
                    generation,
                    camera,
                    viewport: requested_size,
                    mode,
                    refined: false,
                    economy,
                });
                render_pending = true;
                needs_render = false;
            }
            if let Some(completed) = renderer.take_completed() {
                if completed.generation == generation {
                    render_pending = completed.version < requested_version;
                }
                let framebuffer = completed.framebuffer;
                if completed.generation == generation && completed.viewport == requested_size {
                    display_pixels.resize(framebuffer.width * framebuffer.height, 0);
                    for (destination, source) in display_pixels.iter_mut().zip(framebuffer.pixels())
                    {
                        *destination = display_color(*source);
                    }
                    display_size = (framebuffer.width, framebuffer.height);
                    needs_present = true;
                }
            }
            if needs_refine && !render_pending && last_input.elapsed() >= REFINE_DELAY {
                requested_version += 1;
                renderer.request(RenderRequest {
                    version: requested_version,
                    generation,
                    camera,
                    viewport: requested_size,
                    mode,
                    refined: true,
                    economy,
                });
                needs_refine = false;
                render_pending = true;
            }

            if needs_present {
                draw_frame(window, &display_pixels, display_size.0, display_size.1)?;
                needs_present = false;
            }

            if !needs_render && !needs_present {
                if render_pending || needs_refine {
                    // Atiende entrada inmediatamente y recoge cuadros completos sin
                    // bloquear la ventana durante el trazado ni hacer espera activa.
                    unsafe {
                        MsgWaitForMultipleObjects(0, null(), 0, 10, 0x04FF);
                    }
                } else {
                    unsafe { WaitMessage() };
                }
            }
        }

        Ok(())
    }

    fn initial_camera() -> Camera {
        use std::f32::consts::PI;

        Camera::new(
            Vec3::new(11.5, 7.2, 12.5),
            Vec3::new(0.0, 1.45, -9.2),
            Vec3::new(0.0, 1.0, 0.0),
            PI / 3.5,
        )
    }

    fn handle_mode_key(key: usize, mode: &mut RenderMode, economy: &mut RenderOptions) -> bool {
        let selected = match key {
            0x70 => Some(RenderMode::Adaptive), // F1
            0x71 => Some(RenderMode::Quality),  // F2
            0x72 => Some(RenderMode::Economy),  // F3
            _ => None,
        };
        if let Some(selected) = selected {
            if *mode == selected {
                return false;
            }
            *mode = selected;
            return true;
        }
        if *mode != RenderMode::Economy {
            return false;
        }
        let setting = match key {
            0x73 => &mut economy.shadows,
            0x74 => &mut economy.reflections,
            0x75 => &mut economy.refractions,
            0x76 => &mut economy.normal_maps,
            0x77 => &mut economy.antialias,
            _ => return false,
        };
        *setting = !*setting;
        true
    }

    fn show_mode(window: WindowHandle, mode: RenderMode, economy: RenderOptions) {
        let mut title = format!(
            "Templo — {} | F1 Adaptativo | F2 Calidad | F3 Económico",
            mode.name()
        );
        if mode == RenderMode::Economy {
            let state = |enabled| if enabled { "sí" } else { "no" };
            title.push_str(&format!(
                " | F4 Sombras:{} F5 Reflejos:{} F6 Refracción:{} F7 Relieve:{} F8 AA:{}",
                state(economy.shadows),
                state(economy.reflections),
                state(economy.refractions),
                state(economy.normal_maps),
                state(economy.antialias),
            ));
        } else {
            title.push_str(" | Arrastra: rotar | Rueda: zoom | P: guardar");
        }
        println!("{title}");
        unsafe {
            SetWindowTextW(window, wide_string(&title).as_ptr());
        }
    }

    fn handle_key(key: usize, camera: &mut Camera, scene: &Scene) -> io::Result<bool> {
        match key {
            VK_LEFT | 0x41 => camera.orbit(-KEYBOARD_ORBIT_STEP, 0.0),
            VK_RIGHT | 0x44 => camera.orbit(KEYBOARD_ORBIT_STEP, 0.0),
            VK_UP | 0x57 => camera.orbit(0.0, KEYBOARD_ORBIT_STEP),
            VK_DOWN | 0x53 => camera.orbit(0.0, -KEYBOARD_ORBIT_STEP),
            VK_ADD | VK_OEM_PLUS => camera.zoom_proportional(ZOOM_STEP),
            VK_SUBTRACT | VK_OEM_MINUS => camera.zoom_proportional(-ZOOM_STEP),
            0x52 => *camera = initial_camera(),
            0x50 => {
                save_view(scene, camera)?;
                return Ok(false);
            }
            _ => return Ok(false),
        }

        Ok(true)
    }

    fn save_view(scene: &Scene, camera: &Camera) -> io::Result<()> {
        let mut framebuffer = Framebuffer::new(
            SAVED_RENDER_WIDTH,
            SAVED_RENDER_HEIGHT,
            Color::new(8, 10, 18),
        );
        render_refined(&mut framebuffer, camera, scene);
        write_png("window_view.png", &framebuffer)
    }

    fn display_color(color: Color) -> u32 {
        (u32::from(color.r) << 16) | (u32::from(color.g) << 8) | u32::from(color.b)
    }

    fn draw_frame(
        window: WindowHandle,
        pixels: &[u32],
        source_width: usize,
        source_height: usize,
    ) -> io::Result<()> {
        let mut rectangle = Rectangle::default();
        if unsafe { GetClientRect(window, &mut rectangle) } == 0 {
            return Err(io::Error::last_os_error());
        }

        let context = unsafe { GetDC(window) };
        if context.is_null() {
            return Err(io::Error::last_os_error());
        }

        let bitmap_info = BitmapInfo {
            header: BitmapInfoHeader {
                size: size_of::<BitmapInfoHeader>() as u32,
                width: source_width as i32,
                height: -(source_height as i32),
                planes: 1,
                bit_count: 32,
                compression: BI_RGB,
                image_size: (source_width * source_height * 4) as u32,
                x_pixels_per_meter: 0,
                y_pixels_per_meter: 0,
                colors_used: 0,
                important_colors: 0,
            },
            colors: [0],
        };

        unsafe {
            SetStretchBltMode(context, COLORONCOLOR);
            StretchDIBits(
                context,
                0,
                0,
                rectangle.right - rectangle.left,
                rectangle.bottom - rectangle.top,
                0,
                0,
                source_width as i32,
                source_height as i32,
                pixels.as_ptr().cast(),
                &bitmap_info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
            ReleaseDC(window, context);
            ValidateRect(window, null());
        }

        Ok(())
    }

    fn mouse_position(long_parameter: isize) -> (i32, i32) {
        let x = (long_parameter as u16 as i16) as i32;
        let y = ((long_parameter >> 16) as u16 as i16) as i32;
        (x, y)
    }

    fn wide_string(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

#[cfg(target_os = "windows")]
pub use platform::run;

#[cfg(not(target_os = "windows"))]
pub fn run(_scene: &crate::scene::Scene) -> std::io::Result<()> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "El modo de ventana requiere Windows",
    ))
}
