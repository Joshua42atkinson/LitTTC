extends Control

@onready var word_label = $VBoxContainer/WordLabel
@onready var element_label = $VBoxContainer/ElementLabel
@onready var synonyms_label = $VBoxContainer/SynonymsLabel
@onready var pet_preview = $VBoxContainer/PetPreview

func _ready() -> void:
    var word = GameState.active_word
    if word.is_empty():
        word = Database.get_random_word("K-2")
        GameState.active_word = word
        GameState.active_element = Database.get_element(word)

    var element = Database.get_element(word)
    GameState.active_element = element

    word_label.text = word.to_upper()
    element_label.text = "Element: " + element

    var syns = Database.get_synonyms(word)
    synonyms_label.text = "Synonyms: " + ", ".join(syns) if syns.size() > 0 else "(none)"

    pet_preview.color = Database.get_element_color(element)

func _on_summon_pressed() -> void:
    # Generate an enemy based on a random word near the same grade.
    GameState.enemy_word = Database.get_random_word("K-2")
    GameState.enemy_max_health = Database.get_enemy_health_for_word(GameState.enemy_word)
    GameState.enemy_health = GameState.enemy_max_health
    GameState.player_health = 100
    GameState.player_max_health = 100
    get_tree().change_scene_to_file("res://scenes/battle.tscn")
