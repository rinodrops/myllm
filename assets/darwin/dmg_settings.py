import os
import os.path
import unicodedata

# Application bundle — pass via: dmgbuild -D app="dist/darwin-arm64/My App.app"
application = defines.get('app', 'dist/darwin-arm64/My App.app')
# NFD normalization: macOS HFS+ stores filenames in NFD, so icon_locations
# keys must match or dmgbuild silently ignores the position.
appname = unicodedata.normalize('NFD', os.path.basename(application))

# Contents of the DMG
files = [application]
symlinks = {'Applications': '/Applications'}

# Icon positions (logical points, origin = top-left)
icon_locations = {
    appname:        (190, 180),
    'Applications': (480, 180),
}

# Background image (1320x780 @2x for Retina, logical window 660x390)
# Project-specific override wins; falls back to shared template.
_project_bg  = 'assets/darwin/dmg-background.png'
_template_bg = os.path.expanduser('~/.config/templates/dev/dmg-background.png')
background = defines.get(
    'background',
    _project_bg if os.path.exists(_project_bg) else _template_bg
)

# Finder window appearance
show_status_bar  = False
show_tab_view    = False
show_toolbar     = False
show_pathbar     = False
show_sidebar     = False
sidebar_width    = 180

# Window rect: ((x, y from bottom-left of screen), (width, height))  -- screen coords only
window_rect = ((200, 400), (660, 420))

default_view      = 'icon-view'
show_icon_preview = False
icon_size         = 128
text_size         = 13
