use gpui::{Global, Hsla, rgb};

#[derive(Clone, Copy)]
pub struct Theme {
    pub colors: Colors,
}

#[derive(Clone, Copy)]
pub struct Colors {
    pub surface_base: Hsla,
    pub surface_raised: Hsla,
    pub surface_sunken: Hsla,
    pub surface_input: Hsla,

    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_tertiary: Hsla,

    pub border: Hsla,
    pub border_muted: Hsla,
    pub border_focused: Hsla,

    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub accent_active: Hsla,
    pub on_accent: Hsla,

    pub error: Hsla,
    pub warning: Hsla,
    pub success: Hsla,
    pub info: Hsla,
}

impl Global for Theme {}

impl Theme {
    pub fn light() -> Self {
        Self {
            colors: Colors {
                surface_base: rgb(0xeeeeee).into(),
                surface_raised: rgb(0xf4f4f4).into(),
                surface_sunken: rgb(0xe3e3e3).into(),
                surface_input: rgb(0xe9e9e9).into(),

                text_primary: rgb(0x383838).into(),
                text_secondary: rgb(0x666666).into(),
                text_tertiary: rgb(0x9c9c9c).into(),

                border: rgb(0xd0d0d0).into(),
                border_muted: rgb(0xdcdcdc).into(),
                border_focused: rgb(0xa8a8a8).into(),

                accent: rgb(0x2364a8).into(),
                accent_hover: rgb(0x2f78c2).into(),
                accent_active: rgb(0x1b4f85).into(),

                // TODO: this feels funky. Like should foreground be a fn of bg and contrast ratio
                on_accent: rgb(0xffffff).into(),

                error: rgb(0x985050).into(),
                warning: rgb(0x8f744c).into(),
                success: rgb(0x527a56).into(),
                info: rgb(0x58758a).into(),
            },
        }
    }

    pub fn dark() -> Self {
        Self {
            colors: Colors {
                surface_base: rgb(0x1e1e1e).into(),
                surface_raised: rgb(0x262626).into(),
                surface_sunken: rgb(0x161616).into(),
                surface_input: rgb(0x222222).into(),

                text_primary: rgb(0xe8e8e8).into(),
                text_secondary: rgb(0xb5b5b5).into(),
                text_tertiary: rgb(0x858585).into(),

                border: rgb(0x3a3a3a).into(),
                border_muted: rgb(0x2c2c2c).into(),
                border_focused: rgb(0x606060).into(),

                accent: rgb(0x2364a8).into(),
                accent_hover: rgb(0x2f78c2).into(),
                accent_active: rgb(0x1b4f85).into(),
                on_accent: rgb(0xffffff).into(),

                error: rgb(0xd87878).into(),
                warning: rgb(0xc49f68).into(),
                success: rgb(0x86b98a).into(),
                info: rgb(0x88b1ca).into(),
            },
        }
    }
}
