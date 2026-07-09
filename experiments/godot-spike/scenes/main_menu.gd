extends Control

func _on_play_pressed() -> void:
    get_tree().change_scene_to_file("res://scenes/collecting.tscn")

func _on_collection_pressed() -> void:
    get_tree().change_scene_to_file("res://scenes/pet_collection.tscn")

func _on_settings_pressed() -> void:
    get_tree().change_scene_to_file("res://scenes/settings.tscn")
