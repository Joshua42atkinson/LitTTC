import csv
import json
import argparse
import sys
import os

def main():
    parser = argparse.ArgumentParser(description="Import publisher CSV curriculum into word_database.json")
    parser.add_argument("csv_path", help="Path to the input CSV file")
    parser.add_argument("--db-path", default="assets/word_database.json", help="Path to the output JSON database")
    args = parser.parse_args()

    if not os.path.exists(args.csv_path):
        print(f"Error: CSV file '{args.csv_path}' not found.")
        sys.exit(1)

    db_data = {}
    if os.path.exists(args.db_path):
        with open(args.db_path, "r", encoding="utf-8") as f:
            try:
                db_data = json.load(f)
            except json.JSONDecodeError:
                print(f"Warning: Could not parse '{args.db_path}'. Starting fresh.")

    # Default psycholinguistic values if a word is entirely new
    DEFAULT_STATS = {
        "C": 3.0,
        "AoA": 5,
        "V": 5.0,
        "A": 5.0,
        "D": 5.0
    }

    added_count = 0
    updated_count = 0

    with open(args.csv_path, "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        
        # Expected columns: Word, Definition, Grade Level, Common Core Standard
        for row in reader:
            word = row.get("Word", "").strip().lower()
            if not word:
                continue
                
            grade = row.get("Grade Level", "K-12").strip()
            common_core = row.get("Common Core Standard", "").strip()

            if word in db_data:
                db_data[word]["GradeLevel"] = grade
                db_data[word]["CommonCoreStandard"] = common_core
                updated_count += 1
            else:
                db_data[word] = {
                    "C": DEFAULT_STATS["C"],
                    "AoA": DEFAULT_STATS["AoA"],
                    "V": DEFAULT_STATS["V"],
                    "A": DEFAULT_STATS["A"],
                    "D": DEFAULT_STATS["D"],
                    "GradeLevel": grade,
                    "CommonCoreStandard": common_core
                }
                added_count += 1

    # Write back to JSON
    with open(args.db_path, "w", encoding="utf-8") as f:
        json.dump(db_data, f, indent=2)

    print(f"Curriculum imported successfully!")
    print(f"Added new words: {added_count}")
    print(f"Updated existing words: {updated_count}")
    print(f"Total words in DB: {len(db_data)}")

if __name__ == "__main__":
    main()
