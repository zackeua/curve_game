"use strict";

// game_input.js - custom key ytacking and localStorage helpers for curve_game.
//
// Must be included after gl.js and BEFORE load() is called.
// Patches importObject.env with:
//   custom_key_is_down  - checks if an event.key string is currently held
//   custom_get_last_key - pops the most-recently-pressed event.key (any layout)
//   storage_save        - writes a string to localStorage
//   storage_load        - reads a string from localStorage
//
// Keys are event.key values: "a", "A", "å", "ö", "ArrowLeft", " ", "Enter", etc.

var custom_pressed_keys = new Set();
var custom_last_key = null;

canvas.addEventListener("keydown", function (event) {
    custom_pressed_keys.add(event.key);
    custom_last_key = event.key;
});

canvas.addEventListener("keyup", function (event) {
    custom_pressed_keys.delete(event.key);
});

// Patch importObject.env so WASM can call these functions.
// importObject is defined by gl.js before this script runs.
importObject.env.custom_key_is_down = function (ptr, len) {
    var key = UTF8ToString(ptr, len);
    return custom_pressed_keys.has(key) ? 1 : 0;
};

// Returns the length of the key written into buf, or -1 if no key was pressed
// since the last call. The key consumed (cleared) on read.
importObject.env.custom_get_last_key = function (buf_ptr, buf_len) {
    if (custom_last_key === null) return -1;
    var encoded = new TextEncoder().encode(custom_last_key);
    if (encoded.length > buf_len) return -1;
    new Uint8Array(wasm_memory.buffer, buf_ptr, buf_len).set(encoded);
    custom_last_key = null;
    return encoded.length;
};

importObject.env.storage_save = function (key_ptr, key_len, val_ptr, val_len) {
    var key = UTF8ToString(key_ptr, key_len);
    var val = UTF8ToString(val_ptr, val_len);
    try { localStorage.setItem(key, val); } catch (e) {}
};

importObject.env.storage_load = function (key_ptr, key_len, buf_ptr, buf_len) {
    var key = UTF8ToString(key_ptr, key_len);
    var val = localStorage.getItem(key);
    if (val === null) return -1;
    var encoded = new TextEncoder().encode(val);
    if (encoded.length > buf_len) return -1;
    new Uint8Array(wasm_memory.buffer, buf_ptr, buf_len).set(encoded);
    return encoded.length;
};
