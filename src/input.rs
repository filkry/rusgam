use safewindows;

#[derive(Copy, Clone, PartialEq)]
pub enum EInputEdge {
    Unchanged,
    Down,
    Up,
}

#[allow(dead_code)]
pub struct SInput {
    pub a_down: bool,
    pub b_down: bool,
    pub c_down: bool,
    pub d_down: bool,
    pub e_down: bool,
    pub f_down: bool,
    pub g_down: bool,
    pub h_down: bool,
    pub i_down: bool,
    pub j_down: bool,
    pub k_down: bool,
    pub l_down: bool,
    pub m_down: bool,
    pub n_down: bool,
    pub o_down: bool,
    pub p_down: bool,
    pub q_down: bool,
    pub r_down: bool,
    pub s_down: bool,
    pub t_down: bool,
    pub u_down: bool,
    pub v_down: bool,
    pub w_down: bool,
    pub x_down: bool,
    pub y_down: bool,
    pub z_down: bool,

    pub space_down: bool,
    pub tilde_down: bool,
    pub tab_down: bool,
    pub left_arrow_down: bool,
    pub right_arrow_down: bool,
    pub down_arrow_down: bool,
    pub up_arrow_down: bool,
    pub page_up_down: bool,
    pub page_down_down: bool,
    pub home_down: bool,
    pub end_down: bool,
    pub insert_down: bool,
    pub delete_down: bool,
    pub backspace_down: bool,
    pub enter_down: bool,
    pub escape_down: bool,
    pub key_pad_enter_down: bool,
    pub minus_down: bool,

    pub a_edge: EInputEdge,
    pub b_edge: EInputEdge,
    pub c_edge: EInputEdge,
    pub d_edge: EInputEdge,
    pub e_edge: EInputEdge,
    pub f_edge: EInputEdge,
    pub g_edge: EInputEdge,
    pub h_edge: EInputEdge,
    pub i_edge: EInputEdge,
    pub j_edge: EInputEdge,
    pub k_edge: EInputEdge,
    pub l_edge: EInputEdge,
    pub m_edge: EInputEdge,
    pub n_edge: EInputEdge,
    pub o_edge: EInputEdge,
    pub p_edge: EInputEdge,
    pub q_edge: EInputEdge,
    pub r_edge: EInputEdge,
    pub s_edge: EInputEdge,
    pub t_edge: EInputEdge,
    pub u_edge: EInputEdge,
    pub v_edge: EInputEdge,
    pub w_edge: EInputEdge,
    pub x_edge: EInputEdge,
    pub y_edge: EInputEdge,
    pub z_edge: EInputEdge,

    pub space_edge: EInputEdge,
    pub tilde_edge: EInputEdge,
    pub tab_edge: EInputEdge,
    pub left_arrow_edge: EInputEdge,
    pub right_arrow_edge: EInputEdge,
    pub down_arrow_edge: EInputEdge,
    pub up_arrow_edge: EInputEdge,
    pub page_up_edge: EInputEdge,
    pub page_down_edge: EInputEdge,
    pub home_edge: EInputEdge,
    pub end_edge: EInputEdge,
    pub insert_edge: EInputEdge,
    pub delete_edge: EInputEdge,
    pub backspace_edge: EInputEdge,
    pub enter_edge: EInputEdge,
    pub escape_edge: EInputEdge,
    pub key_pad_enter_edge: EInputEdge,
    pub minus_edge: EInputEdge,

    pub left_mouse_down: bool,
    pub middle_mouse_down: bool,
    pub right_mouse_down: bool,
    pub left_mouse_edge: EInputEdge,
    pub middle_mouse_edge: EInputEdge,
    pub right_mouse_edge: EInputEdge,

    pub numbers_down: [bool; 10], // number keys 0-9 pressed
    pub numbers_edge: [EInputEdge; 10],

    pub mouse_dx: i32,
    pub mouse_dy: i32,
}

pub struct SInputEventHandler<'a> {
    input: &'a mut SInput,
    imgui_io: &'a mut imgui::Io,
}

pub fn setup_imgui_key_map(io: &mut imgui::Io) {
    use imgui::{Key};
    use safewindows::{EKey};

    io.key_map[Key::Tab as usize] = EKey::Tab as u32;
    io.key_map[Key::LeftArrow as usize] = EKey::LeftArrow as u32;
    io.key_map[Key::RightArrow as usize] = EKey::RightArrow as u32;
    io.key_map[Key::UpArrow as usize] = EKey::UpArrow as u32;
    io.key_map[Key::DownArrow as usize] = EKey::DownArrow as u32;
    io.key_map[Key::PageUp as usize] = EKey::PageUp as u32;
    io.key_map[Key::PageDown as usize] = EKey::PageDown as u32;
    io.key_map[Key::Home as usize] = EKey::Home as u32;
    io.key_map[Key::End as usize] = EKey::End as u32;
    io.key_map[Key::Insert as usize] = EKey::Insert as u32;
    io.key_map[Key::Delete as usize] = EKey::Delete as u32;
    io.key_map[Key::Backspace as usize] = EKey::Backspace as u32;
    io.key_map[Key::Space as usize] = EKey::Space as u32;
    io.key_map[Key::Enter as usize] = EKey::Enter as u32;
    io.key_map[Key::Escape as usize] = EKey::Escape as u32;
    io.key_map[Key::KeyPadEnter as usize] = EKey::KeyPadEnter as u32;
    io.key_map[Key::A as usize] = EKey::A as u32;
    io.key_map[Key::C as usize] = EKey::C as u32;
    io.key_map[Key::V as usize] = EKey::V as u32;
    io.key_map[Key::X as usize] = EKey::X as u32;
    io.key_map[Key::Y as usize] = EKey::Y as u32;
    io.key_map[Key::Z as usize] = EKey::Z as u32;
}

impl EInputEdge {
    pub fn down(&self) -> bool {
        *self == Self::Down
    }

    #[allow(dead_code)]
    pub fn up(&self) -> bool {
        *self == Self::Up
    }
}

impl SInput {

    pub fn new() -> Self {
        Self {
            a_down: false,
            b_down: false,
            c_down: false,
            d_down: false,
            e_down: false,
            f_down: false,
            g_down: false,
            h_down: false,
            i_down: false,
            j_down: false,
            k_down: false,
            l_down: false,
            m_down: false,
            n_down: false,
            o_down: false,
            p_down: false,
            q_down: false,
            r_down: false,
            s_down: false,
            t_down: false,
            u_down: false,
            v_down: false,
            w_down: false,
            x_down: false,
            y_down: false,
            z_down: false,

            space_down: false,
            tilde_down: false,
            tab_down: false,
            left_arrow_down: false,
            right_arrow_down: false,
            down_arrow_down: false,
            up_arrow_down: false,
            page_up_down: false,
            page_down_down: false,
            home_down: false,
            end_down: false,
            insert_down: false,
            delete_down: false,
            backspace_down: false,
            enter_down: false,
            escape_down: false,
            key_pad_enter_down: false,
            minus_down: false,

            a_edge: EInputEdge::Unchanged,
            b_edge: EInputEdge::Unchanged,
            c_edge: EInputEdge::Unchanged,
            d_edge: EInputEdge::Unchanged,
            e_edge: EInputEdge::Unchanged,
            f_edge: EInputEdge::Unchanged,
            g_edge: EInputEdge::Unchanged,
            h_edge: EInputEdge::Unchanged,
            i_edge: EInputEdge::Unchanged,
            j_edge: EInputEdge::Unchanged,
            k_edge: EInputEdge::Unchanged,
            l_edge: EInputEdge::Unchanged,
            m_edge: EInputEdge::Unchanged,
            n_edge: EInputEdge::Unchanged,
            o_edge: EInputEdge::Unchanged,
            p_edge: EInputEdge::Unchanged,
            q_edge: EInputEdge::Unchanged,
            r_edge: EInputEdge::Unchanged,
            s_edge: EInputEdge::Unchanged,
            t_edge: EInputEdge::Unchanged,
            u_edge: EInputEdge::Unchanged,
            v_edge: EInputEdge::Unchanged,
            w_edge: EInputEdge::Unchanged,
            x_edge: EInputEdge::Unchanged,
            y_edge: EInputEdge::Unchanged,
            z_edge: EInputEdge::Unchanged,

            space_edge: EInputEdge::Unchanged,
            tilde_edge: EInputEdge::Unchanged,
            tab_edge: EInputEdge::Unchanged,
            left_arrow_edge: EInputEdge::Unchanged,
            right_arrow_edge: EInputEdge::Unchanged,
            down_arrow_edge: EInputEdge::Unchanged,
            up_arrow_edge: EInputEdge::Unchanged,
            page_up_edge: EInputEdge::Unchanged,
            page_down_edge: EInputEdge::Unchanged,
            home_edge: EInputEdge::Unchanged,
            end_edge: EInputEdge::Unchanged,
            insert_edge: EInputEdge::Unchanged,
            delete_edge: EInputEdge::Unchanged,
            backspace_edge: EInputEdge::Unchanged,
            enter_edge: EInputEdge::Unchanged,
            escape_edge: EInputEdge::Unchanged,
            key_pad_enter_edge: EInputEdge::Unchanged,
            minus_edge: EInputEdge::Unchanged,

            left_mouse_down: false,
            middle_mouse_down: false,
            right_mouse_down: false,
            left_mouse_edge: EInputEdge::Unchanged,
            middle_mouse_edge: EInputEdge::Unchanged,
            right_mouse_edge: EInputEdge::Unchanged,

            numbers_down: [false; 10],
            numbers_edge: [EInputEdge::Unchanged; 10],

            mouse_dx: 0,
            mouse_dy: 0,
        }
    }

    fn reset_edges(&mut self) {
        self.a_edge = EInputEdge::Unchanged;
        self.b_edge = EInputEdge::Unchanged;
        self.c_edge = EInputEdge::Unchanged;
        self.d_edge = EInputEdge::Unchanged;
        self.e_edge = EInputEdge::Unchanged;
        self.f_edge = EInputEdge::Unchanged;
        self.g_edge = EInputEdge::Unchanged;
        self.h_edge = EInputEdge::Unchanged;
        self.i_edge = EInputEdge::Unchanged;
        self.j_edge = EInputEdge::Unchanged;
        self.k_edge = EInputEdge::Unchanged;
        self.l_edge = EInputEdge::Unchanged;
        self.m_edge = EInputEdge::Unchanged;
        self.n_edge = EInputEdge::Unchanged;
        self.o_edge = EInputEdge::Unchanged;
        self.p_edge = EInputEdge::Unchanged;
        self.q_edge = EInputEdge::Unchanged;
        self.r_edge = EInputEdge::Unchanged;
        self.s_edge = EInputEdge::Unchanged;
        self.t_edge = EInputEdge::Unchanged;
        self.u_edge = EInputEdge::Unchanged;
        self.v_edge = EInputEdge::Unchanged;
        self.w_edge = EInputEdge::Unchanged;
        self.x_edge = EInputEdge::Unchanged;
        self.y_edge = EInputEdge::Unchanged;
        self.z_edge = EInputEdge::Unchanged;
        self.space_edge = EInputEdge::Unchanged;
        self.left_mouse_edge = EInputEdge::Unchanged;
        self.middle_mouse_edge = EInputEdge::Unchanged;
        self.right_mouse_edge = EInputEdge::Unchanged;
    }

    pub fn frame<'a>(&'a mut self, imgui_io: &'a mut imgui::Io) -> SInputEventHandler {
        self.reset_edges();
        SInputEventHandler {
            input: self,
            imgui_io,
        }
    }
}

#[allow(dead_code)]
impl<'a> SInputEventHandler<'a> {
    fn input_change_helper(
        new_down: bool,
        down: &mut bool,
        edge: &mut EInputEdge,
        imgui_io: &mut imgui::Io,
        imgui_io_char: Option<char>,
        imgui_keys_down_idx: Option<usize>,
    ) {
        if new_down && !*down {
            *edge = EInputEdge::Down;
        }
        else if !new_down && *down {
            *edge = EInputEdge::Up;
        }

        *down = new_down;

        if let Some(c) = imgui_io_char {
            if *edge == EInputEdge::Down {
                imgui_io.add_input_character(c);
            }
        }
        if let Some(idx) = imgui_keys_down_idx {
            imgui_io.keys_down[idx] = *down;
        }
    }

    pub fn handle_key_down_up(&mut self, key: safewindows::EKey, down: bool) {
        use safewindows::EKey;

        //let change = &Self::input_change_helper;
        //let i = &mut self.input;

        macro_rules! change {
            ($down_v:ident, $edge_v:ident, $imgui_ch:expr, $imgui_key_down:expr) => {
                Self::input_change_helper(
                    down,
                    &mut self.input.$down_v,
                    &mut self.input.$edge_v,
                    self.imgui_io,
                    $imgui_ch,
                    $imgui_key_down,
                );
            };
        }

        macro_rules! change_number {
            ($number:expr, $imgui_ch:expr) => {
                Self::input_change_helper(
                    down,
                    &mut self.input.numbers_down[$number],
                    &mut self.input.numbers_edge[$number],
                    self.imgui_io,
                    $imgui_ch,
                    None,
                );
            };
        }

        match key {
            EKey::A => change!(a_down, a_edge, Some('a'), Some(EKey::A as usize)),
            EKey::B => change!(b_down, b_edge, Some('b'), None),
            EKey::C => change!(c_down, c_edge, Some('c'), Some(EKey::C as usize)),
            EKey::D => change!(d_down, d_edge, Some('d'), None),
            EKey::E => change!(e_down, e_edge, Some('e'), None),
            EKey::F => change!(f_down, f_edge, Some('f'), None),
            EKey::G => change!(g_down, g_edge, Some('g'), None),
            EKey::H => change!(h_down, h_edge, Some('h'), None),
            EKey::I => change!(i_down, i_edge, Some('i'), None),
            EKey::J => change!(j_down, j_edge, Some('j'), None),
            EKey::K => change!(k_down, k_edge, Some('k'), None),
            EKey::L => change!(l_down, l_edge, Some('l'), None),
            EKey::M => change!(m_down, m_edge, Some('m'), None),
            EKey::N => change!(n_down, n_edge, Some('n'), None),
            EKey::O => change!(o_down, o_edge, Some('o'), None),
            EKey::P => change!(p_down, p_edge, Some('p'), None),
            EKey::Q => change!(q_down, q_edge, Some('q'), None),
            EKey::R => change!(r_down, r_edge, Some('r'), None),
            EKey::S => change!(s_down, s_edge, Some('s'), None),
            EKey::T => change!(t_down, t_edge, Some('t'), None),
            EKey::U => change!(u_down, u_edge, Some('u'), None),
            EKey::V => change!(v_down, v_edge, Some('v'), Some(EKey::V as usize)),
            EKey::W => change!(w_down, w_edge, Some('w'), None),
            EKey::X => change!(x_down, x_edge, Some('x'), Some(EKey::X as usize)),
            EKey::Y => change!(y_down, y_edge, Some('y'), Some(EKey::Y as usize)),
            EKey::Z => change!(z_down, z_edge, Some('z'), Some(EKey::Z as usize)),

            EKey::Number0 => change_number!(0, Some('0')),
            EKey::Number1 => change_number!(1, Some('1')),
            EKey::Number2 => change_number!(2, Some('2')),
            EKey::Number3 => change_number!(3, Some('3')),
            EKey::Number4 => change_number!(4, Some('4')),
            EKey::Number5 => change_number!(5, Some('5')),
            EKey::Number6 => change_number!(6, Some('6')),
            EKey::Number7 => change_number!(7, Some('7')),
            EKey::Number8 => change_number!(8, Some('8')),
            EKey::Number9 => change_number!(9, Some('9')),

            EKey::Tab => change!(tab_down, tab_edge, None, Some(EKey::Tab as usize)),
            EKey::LeftArrow => change!(left_arrow_down, left_arrow_edge, None, Some(EKey::LeftArrow as usize)),
            EKey::RightArrow => change!(right_arrow_down, right_arrow_edge, None, Some(EKey::RightArrow as usize)),
            EKey::UpArrow => change!(up_arrow_down, up_arrow_edge, None, Some(EKey::UpArrow as usize)),
            EKey::DownArrow => change!(down_arrow_down, down_arrow_edge, None, Some(EKey::DownArrow as usize)),
            EKey::PageUp => change!(page_up_down, page_up_edge, None, Some(EKey::PageUp as usize)),
            EKey::PageDown => change!(page_down_down, page_down_edge, None, Some(EKey::PageDown as usize)),
            EKey::Home => change!(home_down, home_edge, None, Some(EKey::Home as usize)),
            EKey::End => change!(end_down, end_edge, None, Some(EKey::End as usize)),
            EKey::Insert => change!(insert_down, insert_edge, None, Some(EKey::Insert as usize)),
            EKey::Delete => change!(delete_down, delete_edge, None, Some(EKey::Delete as usize)),
            EKey::Backspace => change!(backspace_down, backspace_edge, None, Some(EKey::Backspace as usize)),
            EKey::Space => change!(space_down, space_edge, None, Some(EKey::Space as usize)),
            EKey::Enter => change!(enter_down, enter_edge, None, Some(EKey::Enter as usize)),
            EKey::Escape => change!(escape_down, escape_edge, None, Some(EKey::Escape as usize)),
            EKey::KeyPadEnter => change!(key_pad_enter_down, key_pad_enter_edge, None, Some(EKey::KeyPadEnter as usize)),
            EKey::Minus => change!(minus_down, minus_edge, Some('-'), None),
            EKey::Tilde => change!(tilde_down, tilde_edge, Some('~'), None),

            _ => (),
        }
    }

    pub fn handle_lmouse_down_up(&mut self, down: bool) {
        let i = &mut self.input;
        Self::input_change_helper(
            down,
            &mut i.left_mouse_down,
            &mut i.left_mouse_edge,
            self.imgui_io,
            None,
            None);
        self.imgui_io.mouse_down[0] = down;
    }

    pub fn handle_mmouse_down_up(&mut self, down: bool) {
        let i = &mut self.input;
        Self::input_change_helper(
            down,
            &mut i.middle_mouse_down,
            &mut i.middle_mouse_edge,
            self.imgui_io,
            None,
            None);
        self.imgui_io.mouse_down[1] = down;
    }

    pub fn handle_rmouse_down_up(&mut self, down: bool) {
        let i = &mut self.input;
        Self::input_change_helper(
            down,
            &mut i.right_mouse_down,
            &mut i.right_mouse_edge,
            self.imgui_io,
            None,
            None);
        self.imgui_io.mouse_down[2] = down;
    }

    pub fn handle_mouse_move(&mut self, dx: i32, dy: i32) {
        self.input.mouse_dx = dx;
        self.input.mouse_dy = dy;
    }
}

