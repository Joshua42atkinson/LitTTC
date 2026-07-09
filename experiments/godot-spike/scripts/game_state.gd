extends Node

# GameState — global state machine for the 2D 8-bit version.

enum Screen { MAIN_MENU, COLLECTING, SPELLING, BATTLE, QUESTING, COLLECTION, SETTINGS }

var current_screen: Screen = Screen.MAIN_MENU
var active_npc: String = ""
var active_word: String = ""
var active_element: String = ""
var collected_letters: Array = []
var party: Array = []

# Runtime battle state
var player_health: int = 100
var player_max_health: int = 100
var enemy_word: String = ""
var enemy_health: int = 50
var enemy_max_health: int = 50

func change_screen(next: Screen, extra_data: Dictionary = {}) -> void:
    current_screen = next
    if extra_data.has("npc"):
        active_npc = extra_data["npc"]
    if extra_data.has("word"):
        active_word = extra_data["word"]
    emit_signal("screen_changed", next, extra_data)

signal screen_changed(next_screen, data)
