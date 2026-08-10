//! The base16 palette catalog, font sets, and accent slots the themer offers.
//! Pure data; the theme engine (phase 2) will consume the same catalog.

/// A base16 palette: slots 0-7 are surfaces/text dark-to-light, 8-15 accents.
pub struct ThemePalette {
    pub name: &'static str,
    pub dark: [u32; 16],
    /// `None` marks dark-only palettes.
    pub light: Option<[u32; 16]>,
}

impl ThemePalette {
    /// The swatch strip for the requested appearance, falling back to dark.
    pub fn swatches(&self, light: bool) -> &[u32; 16] {
        if light {
            self.light.as_ref().unwrap_or(&self.dark)
        } else {
            &self.dark
        }
    }
}

pub fn catalog() -> &'static [ThemePalette] {
    &[
        ThemePalette {
            name: "Tokyo Night",
            dark: [
                0x1a1b26, 0x24283b, 0x2f3549, 0x444b6a, 0x787c99, 0xa9b1d6, 0xc0caf5, 0xd5d6db,
                0xf7768e, 0xff9e64, 0xe0af68, 0x9ece6a, 0x2ac3de, 0x7aa2f7, 0xbb9af7, 0xd18616,
            ],
            light: Some([
                0xe1e2e7, 0xd4d6e0, 0xc4c8da, 0xa1a6c5, 0x6172b0, 0x3760bf, 0x2e3a5c, 0x006a83,
                0xf52a65, 0xb15c00, 0x8c6c3e, 0x587539, 0x007197, 0x2e7de9, 0x9854f1, 0xc64343,
            ]),
        },
        ThemePalette {
            name: "Gruvbox",
            dark: [
                0x1d2021, 0x3c3836, 0x504945, 0x665c54, 0xbdae93, 0xd5c4a1, 0xebdbb2, 0xfbf1c7,
                0xfb4934, 0xfe8019, 0xfabd2f, 0xb8bb26, 0x8ec07c, 0x83a598, 0xd3869b, 0xd65d0e,
            ],
            light: Some([
                0xf9f5d7, 0xebdbb2, 0xd5c4a1, 0xbdae93, 0x665c54, 0x504945, 0x3c3836, 0x282828,
                0x9d0006, 0xaf3a03, 0xb57614, 0x79740e, 0x427b58, 0x076678, 0x8f3f71, 0xd65d0e,
            ]),
        },
        ThemePalette {
            name: "Nord",
            dark: [
                0x2e3440, 0x3b4252, 0x434c5e, 0x4c566a, 0xd8dee9, 0xe5e9f0, 0xeceff4, 0x8fbcbb,
                0xbf616a, 0xd08770, 0xebcb8b, 0xa3be8c, 0x88c0d0, 0x81a1c1, 0xb48ead, 0x5e81ac,
            ],
            light: None,
        },
        ThemePalette {
            name: "Solarized",
            dark: [
                0x002b36, 0x073642, 0x586e75, 0x657b83, 0x839496, 0x93a1a1, 0xeee8d5, 0xfdf6e3,
                0xdc322f, 0xcb4b16, 0xb58900, 0x859900, 0x2aa198, 0x268bd2, 0x6c71c4, 0xd33682,
            ],
            light: Some([
                0xfdf6e3, 0xeee8d5, 0xd9d2bd, 0x93a1a1, 0x657b83, 0x586e75, 0x073642, 0x002b36,
                0xdc322f, 0xcb4b16, 0xb58900, 0x859900, 0x2aa198, 0x268bd2, 0x6c71c4, 0xd33682,
            ]),
        },
        ThemePalette {
            name: "Catppuccin",
            dark: [
                0x1e1e2e, 0x282839, 0x313244, 0x45475a, 0x585b70, 0xcdd6f4, 0xf5e0dc, 0xb4befe,
                0xf38ba8, 0xfab387, 0xf9e2af, 0xa6e3a1, 0x94e2d5, 0x89b4fa, 0xcba6f7, 0xf2cdcd,
            ],
            light: Some([
                0xeff1f5, 0xe6e9ef, 0xccd0da, 0xbcc0cc, 0x8c8fa1, 0x4c4f69, 0x3c3f54, 0x7287fd,
                0xd20f39, 0xfe640b, 0xdf8e1d, 0x40a02b, 0x179299, 0x1e66f5, 0x8839ef, 0xdd7878,
            ]),
        },
        ThemePalette {
            name: "Rosé Pine",
            dark: [
                0x191724, 0x1f1d2e, 0x26233a, 0x6e6a86, 0x908caa, 0xe0def4, 0xf2f0f7, 0x524f67,
                0xeb6f92, 0xebbcba, 0xf6c177, 0x31748f, 0x9ccfd8, 0x3e8fb0, 0xc4a7e7, 0xea9a97,
            ],
            light: Some([
                0xfaf4ed, 0xf2e9e1, 0xe4dfde, 0x9893a5, 0x797593, 0x575279, 0x26233a, 0xcecacd,
                0xb4637a, 0xd7827e, 0xea9d34, 0x286983, 0x56949f, 0x3e8fb0, 0x907aa9, 0xb4637a,
            ]),
        },
        ThemePalette {
            name: "One",
            dark: [
                0x282c34, 0x353b45, 0x3e4451, 0x545862, 0x8a8f98, 0xabb2bf, 0xc8ccd4, 0xe6e6e6,
                0xe06c75, 0xd19a66, 0xe5c07b, 0x98c379, 0x56b6c2, 0x61afef, 0xc678dd, 0xbe5046,
            ],
            light: Some([
                0xfafafa, 0xf0f0f1, 0xe5e5e6, 0xa0a1a7, 0x696c77, 0x383a42, 0x202227, 0x090a0b,
                0xca1243, 0xd75f00, 0xc18401, 0x50a14f, 0x0184bc, 0x4078f2, 0xa626a4, 0x986801,
            ]),
        },
        ThemePalette {
            name: "Kanagawa",
            dark: [
                0x1f1f28, 0x2a2a37, 0x363646, 0x54546d, 0x727169, 0xdcd7ba, 0xc8c093, 0x717c7c,
                0xc34043, 0xffa066, 0xdca561, 0x98bb6c, 0x7aa89f, 0x7e9cd8, 0x957fb8, 0xd27e99,
            ],
            light: Some([
                0xf2ecbc, 0xe5ddb0, 0xdcd5a0, 0x8a8980, 0x716e61, 0x545464, 0x43436c, 0x22262d,
                0xc84053, 0xcc6d00, 0xa8801f, 0x6f894e, 0x597b75, 0x4d699b, 0xb35b79, 0xa09a7f,
            ]),
        },
    ]
}

/// The base16 slots offered as accent choices.
pub const ACCENT_SLOTS: [usize; 6] = [8, 10, 11, 12, 13, 14];

pub struct FontSet {
    pub label: &'static str,
    pub sample: &'static str,
    pub ui_family: &'static str,
    pub mono_family: &'static str,
}

pub fn font_sets() -> &'static [FontSet] {
    &[
        FontSet {
            label: "Plex",
            sample: "mono 0123",
            ui_family: "IBM Plex Sans",
            mono_family: "JetBrains Mono",
        },
        FontSet {
            label: "Neue",
            sample: "mono 0123",
            ui_family: "Helvetica Neue",
            mono_family: "JetBrains Mono",
        },
        FontSet {
            label: "Mono",
            sample: "mono 0123",
            ui_family: "JetBrains Mono",
            mono_family: "JetBrains Mono",
        },
    ]
}
