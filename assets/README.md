# Assets Folder

This folder contains all the icons and images used by the FFmpeg Rust application.

## Icon Requirements

The application expects the following icon files in this folder:

### Required Icons (16x16 or 24x24 PNG format recommended)

- `folder.png` - Folder icon for file/directory selection
- `file.png` - File icon for input file display
- `play.png` - Play/start conversion icon
- `stop.png` - Stop conversion icon
- `settings.png` - Settings/preferences icon
- `help.png` - Help/question mark icon
- `update.png` - Update/download icon
- `save.png` - Save preset icon
- `load.png` - Load preset icon
- `delete.png` - Delete preset icon

### Optional Icons

- `convert.png` - Convert mode icon
- `remux.png` - Remux mode icon
- `video.png` - Video codec icon
- `audio.png` - Audio codec icon
- `progress.png` - Progress indicator icon

## Icon Style Guidelines

- Use simple, minimalistic design
- Monochrome or subtle colors preferred
- 16x16 or 24x24 pixels for optimal display
- PNG format with transparency support
- Dark theme compatible (light colored icons work best)

## Icon Sources

You can obtain suitable icons from:
- [Feather Icons](https://feathericons.com/) - Minimalistic line icons
- [Heroicons](https://heroicons.com/) - Simple solid and outline icons
- [Lucide](https://lucide.dev/) - Beautiful and consistent icons
- [Phosphor Icons](https://phosphoricons.com/) - Flexible icon family

## Adding Icons

1. Save icon files in this folder with the exact names listed above
2. Ensure icons are properly sized and formatted
3. Test icons in both light and dark themes
4. The application will automatically load icons from this folder

## Fallback Behavior

If icon files are missing, the application will:
- Use Unicode symbols as fallbacks (📁, 📄, ▶️, ⏹️, etc.)
- Continue to function normally without visual icons
- Display text labels instead of missing icons