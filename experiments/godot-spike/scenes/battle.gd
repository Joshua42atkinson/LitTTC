extends Control

@onready var player_word = $VBoxContainer/BarsContainer/PlayerColumn/PlayerWord
@onready var player_health = $VBoxContainer/BarsContainer/PlayerColumn/PlayerHealth
@onready var player_pet = $VBoxContainer/BarsContainer/PlayerColumn/PlayerPet
@onready var enemy_word = $VBoxContainer/BarsContainer/EnemyColumn/EnemyWord
@onready var enemy_health = $VBoxContainer/BarsContainer/EnemyColumn/EnemyHealth
@onready var enemy_pet = $VBoxContainer/BarsContainer/EnemyColumn/EnemyPet
@onready var attack_button = $VBoxContainer/AttackButton
@onready var result_label = $VBoxContainer/ResultLabel

func _ready() -> void:
    randomize()
    if GameState.active_word.is_empty():
        GameState.active_word = Database.get_random_word("K-2")
        GameState.active_element = Database.get_element(GameState.active_word)
    if GameState.enemy_word.is_empty():
        GameState.enemy_word = Database.get_random_word("K-2")
        GameState.enemy_max_health = Database.get_enemy_health_for_word(GameState.enemy_word)
        GameState.enemy_health = GameState.enemy_max_health
        GameState.player_health = GameState.player_max_health

    player_word.text = GameState.active_word.to_upper()
    enemy_word.text = GameState.enemy_word.to_upper()

    player_pet.color = Database.get_element_color(GameState.active_element)
    enemy_pet.color = Color(0.9, 0.1, 0.1)
    _update_ui()

func _update_ui() -> void:
    player_health.text = "HP: %d/%d" % [GameState.player_health, GameState.player_max_health]
    enemy_health.text = "HP: %d/%d" % [GameState.enemy_health, GameState.enemy_max_health]

func _on_attack_pressed() -> void:
    attack_button.disabled = true
    var damage = Database.get_word_damage(GameState.active_word)
    GameState.enemy_health = maxi(GameState.enemy_health - damage, 0)
    _update_ui()

    if GameState.enemy_health <= 0:
        result_label.text = "You vanquished the typo!"
        await get_tree().create_timer(1.0).timeout
        get_tree().change_scene_to_file("res://scenes/collecting.tscn")
        return

    var enemy_damage = Database.get_word_damage(GameState.enemy_word)
    GameState.player_health = maxi(GameState.player_health - enemy_damage, 0)
    _update_ui()

    if GameState.player_health <= 0:
        result_label.text = "Your pet fainted..."
        await get_tree().create_timer(1.5).timeout
        get_tree().change_scene_to_file("res://scenes/main_menu.tscn")
        return

    attack_button.disabled = false
