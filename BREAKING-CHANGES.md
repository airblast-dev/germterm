# 0.2.0
- Drawing functions now take a `&mut Layer` as the first argument instead of `&mut Engine` to account for the new layer system

# 0.2.1
- Changed visibility of `fps_counter::FpsCounter` from `pub` to `pub(crate)`. Please use the new `fps_counter::get_fps` function if you need to read the current FPS.
- Moved `fps_counter::draw_fps_counter` to `draw::draw_fps_counter`.
