// 창을 띄우는 쪽은 이게 전부다. 판정은 전부 프론트엔드(ui/index.html)에 있다.
//
// 투명·테두리 없음·항상 위는 tauri.conf.json의 설정이고, 클릭 통과만 구현이다.
// 그래서 Rust 쪽에는 넣을 것이 없다.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("failed to start the overlay window example");
}
