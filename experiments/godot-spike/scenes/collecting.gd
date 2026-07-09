extends Control

@onready var word_list = $VBoxContainer/WordsContainer
@onready var title_label = $VBoxContainer/Title

func _ready() -> void:
    randomize()
    var words = _pick_words(3)
    for word in words:
        var btn = Button.new()
        btn.text = word
        btn.pressed.connect(_on_word_chosen.bind(word))
        word_list.add_child(btn)

func _pick_words(count: int) -> Array:
    var result: Array = []
    for i in range(count):
        result.append(Database.get_random_word("K-2"))
    return result

func _on_word_chosen(word: String) -> void:
    GameState.active_word = word
    GameState.active_element = Database.get_element(word)
    get_tree().change_scene_to_file("res://scenes/spelling.tscn")
