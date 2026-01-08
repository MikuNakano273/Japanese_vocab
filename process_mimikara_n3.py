#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
process_mimikara_n3.py

Helper script that:
- Parses the raw `mimikara_n3.txt` file and writes a normalized,
  numbered fixed file `mimikara_n3_fixed.txt`.
  - Removes any 'nan' tokens.
  - For lines with two fields where the first field is kana-only,
    treats them as (kana, meaning) and sets kanji empty.
  - For lines with three fields, uses them as (kanji, kana, meaning).
- Invokes the question-generation functions from `convert_kana_entries.py`
  to build the SQLite DB of multiple-choice questions.

This script is intended to live in the same directory as:
- `mimikara_n3.txt` (input)
- `convert_kana_entries.py` (module providing generation utilities)

Usage:
    python process_mimikara_n3.py [--input mimikara_n3.txt] [--fixed mimikara_n3_fixed.txt]
                                  [--db mimikara_n3_questions.db] [--seed N] [--force] [--show-sample]

Options:
    --input / -i    Path to raw input file (default: mimikara_n3.txt)
    --fixed / -f    Path to write fixed numbered file (default: mimikara_n3_fixed.txt)
    --db / -d       Path to output SQLite DB (default: mimikara_n3_questions.db)
    --seed          Optional int seed for deterministic sampling
    --force         Overwrite DB if exists
    --show-sample   Print sample questions saved to DB

Note: This script imports and uses functions from convert_kana_entries.py.
If that module is not present or import fails, this script will report an error.
"""

from __future__ import annotations

import argparse
import json
import re
import sqlite3
import sys
from pathlib import Path
from typing import List, Optional, Tuple

# Attempt to import convert_kana_entries module (must be in same directory)
try:
    import convert_kana_entries as cke
except Exception as exc:  # pragma: no cover - runtime import error handling
    cke = None
    _IMPORT_ERROR = exc


# Kana detection helper (hiragana + katakana + prolonged vowel and middle dot)
def _is_kana_only(s: str) -> bool:
    s = s.strip()
    if not s:
        return False
    # Hiragana \u3040-\u309F, Katakana \u30A0-\u30FF, Katakana ext \u31F0-\u31FF,
    # prolonged sound mark \u30FC, middle dot \u30FB, iteration marks \u309D\u309E
    kana_re = re.compile(
        r"^[\u3040-\u309F\u30A0-\u30FF\u31F0-\u31FF\u30FC\u30FB\u309D\u309E\s]+$"
    )
    return bool(kana_re.fullmatch(s))


def parse_raw_file(path: Path) -> List[Tuple[str, str, str]]:
    """
    Parse the raw mimikara_n3.txt file and return a list of tuples:
      (kanji, kana, meaning)
    Rules applied:
      - Lines that don't contain a comma are ignored.
      - Trailing commas trimmed.
      - Token 'nan' (case-insensitive) removed from parts.
      - If 3+ parts after cleanup: use first three as (kanji, kana, meaning).
      - If exactly 2 parts:
          * If first part is kana-only -> (kanji='', kana=first, meaning=second)
          * Else -> (kanji=first, kana=second, meaning='')
      - If exactly 1 part:
          * If part is kana-only -> kana
          * Else -> meaning
    """
    if not path.exists():
        raise FileNotFoundError(f"Input file not found: {path}")

    entries: List[Tuple[str, str, str]] = []
    with path.open("r", encoding="utf-8") as fh:
        for raw in fh:
            line = raw.strip()
            if not line:
                continue
            # skip lines that look like section headers (no comma)
            if "," not in line:
                continue
            # remove trailing commas
            line = line.rstrip(",")
            # split into at most 3 parts so meaning may contain commas
            parts = [p.strip() for p in line.split(",", 2)]
            # remove any 'nan' tokens (case-insensitive)
            parts = [p for p in parts if p and p.lower() != "nan"]
            kanji = ""
            kana = ""
            meaning = ""
            if len(parts) >= 3:
                kanji, kana, meaning = parts[0], parts[1], parts[2]
            elif len(parts) == 2:
                a, b = parts[0], parts[1]
                if _is_kana_only(a):
                    kanji = ""
                    kana = a
                    meaning = b
                else:
                    kanji = a
                    kana = b
                    meaning = ""
            elif len(parts) == 1:
                a = parts[0]
                if _is_kana_only(a):
                    kana = a
                else:
                    meaning = a
            else:
                continue
            entries.append((kanji, kana, meaning))
    return entries


def write_fixed_file(entries: List[Tuple[str, str, str]], out_path: Path) -> None:
    """
    Write numbered fixed file with lines like:
      1.   kanji, kana, meaning
    kanji/kana/meaning may be empty strings.
    """
    with out_path.open("w", encoding="utf-8", newline="\n") as fh:
        for i, (kanji, kana, meaning) in enumerate(entries, start=1):
            # ensure no leading/trailing accidental commas
            kanji_s = kanji.strip()
            kana_s = kana.strip()
            meaning_s = meaning.strip()
            fh.write(f"{i}.   {kanji_s}, {kana_s}, {meaning_s}\n")


def run_generation(
    fixed_path: Path, db_path: Path, seed: int | None, force: bool, show_sample: bool
) -> None:
    """
    Use functions from convert_kana_entries module to parse fixed file and
    generate/save question DB. This function expects convert_kana_entries to
    be available and exposes high-level operations.
    """
    if cke is None:
        raise ImportError(
            f"Required module convert_kana_entries could not be imported: {_IMPORT_ERROR!r}"
        )

    # Use the module's parser to read the fixed file (keeps consistent parsing)
    entries = cke.parse_fixed_file(fixed_path)
    if not entries:
        print("No entries parsed from fixed file; aborting generation.")
        return

    # Optionally set seed for reproducibility
    if seed is not None:
        import random

        random.seed(seed)

    kanji_pool, kana_pool, meaning_pool = cke.build_pools(entries)
    print(f"Parsed {len(entries)} entries from fixed file.")
    print(
        f"Pools sizes -> kanji: {len(kanji_pool)}, kana: {len(kana_pool)}, meaning: {len(meaning_pool)}"
    )

    # Generate questions (this returns question dicts and skipped count)
    questions, skipped = cke.generate_all_questions(
        entries, kanji_pool, kana_pool, meaning_pool
    )
    print(
        f"Generated {len(questions)} questions; skipped {skipped} due to insufficient distractors."
    )

    # Create DB and insert entries+questions
    conn = cke.create_db(db_path, force=force)
    try:
        entry_db_ids = cke.insert_entries(conn, entries)
        cke.insert_questions(conn, questions, entry_db_ids)
    finally:
        conn.close()

    print(
        f"Saved entries and {len(questions)} questions into DB: {db_path} (overwrite={force})"
    )

    # Optionally print sample
    if show_sample:
        import json
        import sqlite3

        conn2 = sqlite3.connect(str(db_path))
        cur = conn2.cursor()
        cur.execute(
            "SELECT id, q_type, prompt, correct_answer, options, correct_index FROM questions ORDER BY id LIMIT 20"
        )
        rows = cur.fetchall()
        print("\nSample saved questions:")
        for r in rows:
            qid, qtype, prompt, correct_answer, options_json, correct_index = r
            options = json.loads(options_json)
            print(f"QID {qid} (type={qtype}): {prompt}")
            for i, opt in enumerate(options):
                mark = " (correct)" if i == correct_index else ""
                print(f"  {i + 1}. {opt}{mark}")
            print()
        conn2.close()


def _ensure_questions_columns(conn: sqlite3.Connection) -> None:
    """
    Ensure 'level' and 'chapter' columns exist on 'questions' table.
    Adds columns if missing.
    """
    cur = conn.cursor()
    cur.execute("PRAGMA table_info(questions)")
    cols = [r[1] for r in cur.fetchall()]
    if "level" not in cols:
        cur.execute("ALTER TABLE questions ADD COLUMN level INTEGER")
    if "chapter" not in cols:
        cur.execute("ALTER TABLE questions ADD COLUMN chapter INTEGER")
    conn.commit()


def _ensure_entries_columns(conn: sqlite3.Connection) -> None:
    """
    Ensure 'level' and 'chapter' columns exist on 'entries' table.
    Adds columns if missing.
    """
    cur = conn.cursor()
    cur.execute("PRAGMA table_info(entries)")
    cols = [r[1] for r in cur.fetchall()]
    if "level" not in cols:
        cur.execute("ALTER TABLE entries ADD COLUMN level INTEGER")
    if "chapter" not in cols:
        cur.execute("ALTER TABLE entries ADD COLUMN chapter INTEGER")
    conn.commit()


def apply_chapters(db_path: Path, fixed_path: Path, level_str: str = "n4") -> None:
    """
    Read the fixed numbered file and apply chapter information to the
    entries and questions tables in the SQLite DB. Also creates/updates n_level table.

    - fixed_path: Path to the fixed numbered file (e.g. mimikara_n3_numbered_fixed.txt).
    - level_str: one of n5, n4, n3, n2, n1 (default n4). Will be mapped to id 1..5
                 where id 1 -> n5, id 5 -> n1 (per your spec).

    Behavior:
      - Create/populate `n_level` with canonical mapping (1..5 -> n5..n1).
      - Ensure `entries` has `level` and `chapter` columns and `questions` has `level` and `chapter`.
      - Set `level` for all rows in `entries` and `questions` to the chosen level_id.
      - Parse numbered fixed file: each { ... } block is a chapter; map entry numbers to chapter indices.
      - Update `entries.chapter` for those entry numbers, then copy `entries.chapter` into `questions.chapter`.
    """
    level_map = {"n5": 1, "n4": 2, "n3": 3, "n2": 4, "n1": 5}
    level_key = level_str.lower()
    if level_key not in level_map:
        raise ValueError(
            f"Invalid level '{level_str}'; expected one of {list(level_map.keys())}"
        )
    level_id = level_map[level_key]

    db_file = Path(db_path)
    if not db_file.exists():
        raise FileNotFoundError(f"Database file not found: {db_file}")

    # Read fixed file and extract chapters: each {...} block is a chapter
    text = fixed_path.read_text(encoding="utf-8")
    # Find blocks between braces { ... }
    chapter_blocks = re.findall(r"\{([^}]*)\}", text, flags=re.S)
    if not chapter_blocks:
        # If no braces found, consider the entire file a single chapter
        chapter_blocks = [text]

    # Build mapping entry_number -> chapter_index (1-based)
    entry_to_chapter: dict[int, int] = {}
    for ci, block in enumerate(chapter_blocks, start=1):
        for raw_line in block.splitlines():
            line = raw_line.strip()
            if not line:
                continue
            # Expect lines like "123.   ...", so capture leading number
            m = re.match(r"^\s*(\d+)\.", line)
            if m:
                try:
                    entry_num = int(m.group(1))
                    entry_to_chapter[entry_num] = ci
                except ValueError:
                    continue

    conn = sqlite3.connect(str(db_file))
    try:
        cur = conn.cursor()
        # create n_level table and populate with 5 rows mapping n5..n1 to ids 1..5
        cur.execute(
            "CREATE TABLE IF NOT EXISTS n_level (id INTEGER PRIMARY KEY, level TEXT NOT NULL)"
        )
        # insert or replace the canonical five rows
        for name, idx in [("n5", 1), ("n4", 2), ("n3", 3), ("n2", 4), ("n1", 5)]:
            cur.execute(
                "INSERT OR REPLACE INTO n_level (id, level) VALUES (?, ?)",
                (idx, name),
            )
        conn.commit()

        # Ensure entries and questions tables have required columns
        _ensure_entries_columns(conn)
        _ensure_questions_columns(conn)

        # Set level on all entries (if table exists) and on all questions
        cur.execute(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='entries'"
        )
        entries_exist = bool(cur.fetchone())
        entries_updated = 0
        if entries_exist:
            cur.execute("UPDATE entries SET level = ?", (level_id,))
            entries_updated = cur.rowcount

        cur.execute("UPDATE questions SET level = ?", (level_id,))
        questions_level_updated = cur.rowcount
        conn.commit()

        # Apply chapter indices to entries (if entries table exists)
        entries_chapter_updated = 0
        if entries_exist and entry_to_chapter:
            for entry_num, chapter_idx in entry_to_chapter.items():
                # Only update if that entry id exists in entries
                cur.execute(
                    "UPDATE entries SET chapter = ? WHERE id = ?",
                    (chapter_idx, entry_num),
                )
                entries_chapter_updated += cur.rowcount
            conn.commit()

            # Propagate chapters from entries into questions
            # Set questions.chapter to the entry's chapter where entry_id matches entries.id
            cur.execute(
                "UPDATE questions SET chapter = (SELECT chapter FROM entries WHERE entries.id = questions.entry_id)"
            )
            questions_chapter_updated = cur.rowcount
            conn.commit()
        else:
            questions_chapter_updated = 0

        print(
            f"n_level ensured. entries_exist={entries_exist}, entries_level_updated={entries_updated}, "
            f"questions_level_updated={questions_level_updated}, entries_chapter_updated={entries_chapter_updated}, "
            f"questions_chapter_updated={questions_chapter_updated} (applied level_id={level_id})."
        )
    finally:
        conn.close()


def read_level_from_txt(path: Path) -> int:
    """
    Read first non-empty line of txt file.
    Expect format: n5, n4, n3, n2, n1
    Return mapped level id (1..5)
    """
    level_map = {"n5": 1, "n4": 2, "n3": 3, "n2": 4, "n1": 5}

    with path.open("r", encoding="utf-8") as fh:
        for line in fh:
            line = line.strip().lower()
            if not line:
                continue
            if line in level_map:
                return level_map[line]
            break

    raise ValueError("Level header (n5..n1) not found at top of txt file")


def parse_chapters_from_fixed(fixed_path: Path) -> dict[int, int]:
    """
    Each { ... } block is one chapter (1-based).
    Return mapping: entry_id -> chapter_number
    """
    text = fixed_path.read_text(encoding="utf-8")
    blocks = re.findall(r"\{([^}]*)\}", text, flags=re.S)

    if not blocks:
        blocks = [text]

    mapping: dict[int, int] = {}

    for chapter_idx, block in enumerate(blocks, start=1):
        for line in block.splitlines():
            line = line.strip()
            if not line:
                continue
            m = re.match(r"^(\d+)\.", line)
            if m:
                entry_id = int(m.group(1))
                mapping[entry_id] = chapter_idx

    return mapping


def apply_level_and_chapters_from_txt(
    db_path: Path,
    fixed_path: Path,
):
    level_id = read_level_from_txt(fixed_path)
    entry_to_chapter = parse_chapters_from_fixed(fixed_path)

    conn = sqlite3.connect(str(db_path))
    try:
        cur = conn.cursor()

        # n_level table
        cur.execute(
            "CREATE TABLE IF NOT EXISTS n_level (id INTEGER PRIMARY KEY, level TEXT NOT NULL)"
        )
        for name, idx in [("n5", 1), ("n4", 2), ("n3", 3), ("n2", 4), ("n1", 5)]:
            cur.execute(
                "INSERT OR REPLACE INTO n_level (id, level) VALUES (?, ?)",
                (idx, name),
            )

        _ensure_entries_columns(conn)
        _ensure_questions_columns(conn)

        # set level for all
        cur.execute("UPDATE entries SET level = ?", (level_id,))
        cur.execute("UPDATE questions SET level = ?", (level_id,))
        conn.commit()

        # set chapter for entries (ID = số thứ tự)
        for entry_id, chapter in entry_to_chapter.items():
            cur.execute(
                "UPDATE entries SET chapter = ? WHERE id = ?",
                (chapter, entry_id),
            )
        conn.commit()

        # propagate chapter to questions
        cur.execute(
            """
            UPDATE questions
            SET chapter = (
                SELECT chapter
                FROM entries
                WHERE entries.id = questions.entry_id
            )
            """
        )
        conn.commit()

        print(
            f"Applied level={level_id}, chapters={len(set(entry_to_chapter.values()))}"
        )

    finally:
        conn.close()


def main(argv=None):
    parser = argparse.ArgumentParser(
        description="Process raw mimikara file, normalize and generate MC questions DB."
    )
    parser.add_argument(
        "--input",
        "-i",
        type=Path,
        default=Path("mimikara_n3.txt"),
        help="Raw input file path (default: mimikara_n3.txt)",
    )
    parser.add_argument(
        "--fixed",
        "-f",
        type=Path,
        default=Path("mimikara_n3_fixed.txt"),
        help="Fixed output file path (default: mimikara_n3_fixed.txt)",
    )
    parser.add_argument(
        "--db",
        "-d",
        type=Path,
        default=Path("mimikara_n3_questions.db"),
        help="Output SQLite DB path (default: mimikara_n3_questions.db)",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=None,
        help="Optional random seed for deterministic sampling",
    )
    parser.add_argument(
        "--force", action="store_true", help="Overwrite DB if it exists"
    )
    parser.add_argument(
        "--show-sample",
        action="store_true",
        help="Print a few sample questions after generation",
    )
    parser.add_argument(
        "--apply-chapters",
        action="store_true",
        help="Read fixed numbered file and apply chapter/level info into the DB (creates n_level table).",
    )
    parser.add_argument(
        "--level",
        type=str,
        default="n4",
        help="Level label to apply when using --apply-chapters (n5..n1). Default: n4",
    )
    parser.add_argument(
        "--fixed-chapters",
        type=Path,
        default=Path("mimikara_n3_numbered_fixed.txt"),
        help="Fixed numbered file to read chapters from (default: mimikara_n3_numbered_fixed.txt).",
    )

    args = parser.parse_args(argv)

    # If user only wants to apply chapters, do that and exit
    if args.apply_chapters:
        try:
            # Use the TXT-driven apply function which reads the level header
            # from the fixed/chapter file and assigns chapter numbers per { ... } blocks.
            apply_level_and_chapters_from_txt(args.db, args.fixed_chapters)
        except Exception as exc:
            print(f"Applying chapters failed: {exc}", file=sys.stderr)
            sys.exit(5)
        print("Chapter information applied successfully.")
        return

    try:
        entries = parse_raw_file(args.input)
    except Exception as exc:
        print(f"Failed to parse input file {args.input}: {exc}", file=sys.stderr)
        sys.exit(2)

    if not entries:
        print("No entries were parsed from the input file. Exiting.", file=sys.stderr)
        sys.exit(1)

    try:
        write_fixed_file(entries, args.fixed)
    except Exception as exc:
        print(f"Failed to write fixed file {args.fixed}: {exc}", file=sys.stderr)
        sys.exit(3)

    print(f"Wrote fixed file with {len(entries)} entries to: {args.fixed}")

    try:
        run_generation(args.fixed, args.db, args.seed, args.force, args.show_sample)
    except Exception as exc:
        print(f"Generation failed: {exc}", file=sys.stderr)
        sys.exit(4)

    try:
        # After generation, apply level & chapter metadata using the numbered fixed file
        # specified by --fixed-chapters (not the generation fixed output).
        apply_level_and_chapters_from_txt(args.db, args.fixed_chapters)
    except Exception as exc:
        print(f"Skip auto level/chapter apply: {exc}")


if __name__ == "__main__":
    main()
