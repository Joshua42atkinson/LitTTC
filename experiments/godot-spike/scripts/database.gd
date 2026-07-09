extends Node

# Database — loads LitTCG JSON data at runtime.
# Reads from res://assets/data/ (symlinked to LitTTC/assets).

var words: Dictionary = {}
var synonyms: Dictionary = {}
var etymology: Dictionary = {}
var quests: Dictionary = {}
var npcs: Dictionary = {}

var _loaded: bool = false

func _ready() -> void:
    load_all()

func load_all() -> void:
    if _loaded:
        return
    words = _load_json("res://assets/data/word_database.json")
    synonyms = _load_json("res://assets/data/synonym_database.json")
    etymology = _load_json("res://assets/data/etymology_db.json")
    quests = _load_json("res://assets/data/quest_data.json")
    npcs = _load_json("res://assets/data/lore_db.json")
    _loaded = true
    print("Database loaded: %d words, %d synonyms, %d etymology roots, %d NPCs" % [
        words.size(), synonyms.size(), etymology.get("Roots", {}).size(), npcs.size()
    ])

func _load_json(path: String) -> Dictionary:
    var file = FileAccess.open(path, FileAccess.READ)
    if file == null:
        push_error("Failed to open %s: %s" % [path, error_string(FileAccess.get_open_error())])
        return {}
    var text = file.get_as_text()
    file.close()
    var parsed = JSON.parse_string(text)
    if parsed == null:
        push_error("Failed to parse JSON: %s" % path)
        return {}
    if parsed is Dictionary:
        return parsed as Dictionary
    if parsed is Array:
        var wrapped = {}
        for i in range(parsed.size()):
            wrapped[str(i)] = parsed[i]
        return wrapped
    return {}

func has_word(word: String) -> bool:
    return words.has(word)

func get_word_stats(word: String) -> Dictionary:
    return words.get(word, {})

func get_synonyms(word: String) -> Array:
    var entry = synonyms.get(word, {})
    return entry.get("Synonyms", [])

func get_element(word: String) -> String:
    var entry = synonyms.get(word, {})
    return entry.get("Element", "")

func get_random_word(min_grade: String = "K-2") -> String:
    var pool: Array = []
    for w in words.keys():
        var stats = words[w]
        if stats.get("GradeLevel", "") == min_grade:
            pool.append(w)
    if pool.is_empty():
        # Fall back to all words
        pool = words.keys()
    if pool.is_empty():
        return "spell"
    return pool[randi() % pool.size()]

func get_element_color(element: String) -> Color:
    match element.to_lower():
        "fire": return Color(0.9, 0.2, 0.1)
        "water": return Color(0.1, 0.4, 0.9)
        "earth": return Color(0.2, 0.6, 0.1)
        "air": return Color(0.6, 0.8, 0.9)
        "light": return Color(0.95, 0.9, 0.4)
        "dark": return Color(0.3, 0.1, 0.4)
        _:
            return Color(0.5, 0.5, 0.5)

func get_word_damage(word: String) -> int:
    var stats = get_word_stats(word)
    var c = stats.get("C", 2.5)
    var v = stats.get("V", 5.0)
    var a = stats.get("A", 5.0)
    var d = stats.get("D", 5.0)
    var base = (c + v + a + d) * 3.0
    return clampi(int(round(base)), 1, 50)

func get_enemy_health_for_word(word: String) -> int:
    var stats = get_word_stats(word)
    var concreteness = stats.get("C", 2.5)
    return clampi(int(round(concreteness * 15.0)), 10, 80)
