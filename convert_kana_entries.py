#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
convert_kana_entries.py

Lightweight implementation to:
- Parse a fixed numbered file of entries (lines like "1. kanji, kana, meaning")
- Build simple pools for distractors
- Generate multiple-choice questions (one per entry: kana -> meaning)
- Create an SQLite DB and insert `entries` and `questions` tables/rows

This module exposes the functions expected by `process_mimikara_n3.py`:
- parse_fixed_file(path: Path) -> List[Tuple[str,str,str]]
- build_pools(entries) -> (kanji_pool, kana_pool, meaning_pool)
- generate_all_questions(entries, kanji_pool, kana_pool, meaning_pool) -> (questions, skipped)
- create_db(db_path: Path, force: bool) -> sqlite3.Connection
- insert_entries(conn, entries) -> List[int]  # list of inserted entry ids
- insert_questions(conn, questions, entry_db_ids) -> int  # number inserted

Notes:
- Questions generated: prompt is the `kana` and the question asks for the meaning.
- Options are a list of 4 strings: one correct meaning and up to 3 distractors sampled
  from other meanings in the pool. If not enough distractors found, the question is skipped.
- The DB schema is conservative and compatible with the backend init:
  tables created: entries, questions (superset), n_level (not populated here),
  tests, quizzes (if not exist). This function will not drop tables unless `force` is True.
"""

from __future__ import annotations

import json
import random
import re
import sqlite3
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# ---- Parsing utilities ----

_ENTRY_LINE_RE = re.compile(r"^\s*(\d+)\.\s*(.*)$")


def parse_fixed_file(path: Path) -> List[Tuple[str, str, str]]:
    """
    Parse a fixed numbered file and return list of (kanji, kana, meaning).
    Expected line example:
      123.  漢字, かんじ, meaning here
    Lines not matching are skipped.
    """
    if not path.exists():
        raise FileNotFoundError(f"Fixed file not found: {path}")

    entries: List[Tuple[str, str, str]] = []
    with path.open("r", encoding="utf-8") as fh:
        for raw in fh:
            line = raw.strip()
            if not line:
                continue
            m = _ENTRY_LINE_RE.match(line)
            if not m:
                # skip lines that don't start with "N. "
                continue
            rest = m.group(2).strip()
            # split into up to 3 parts (kanji, kana, meaning)
            parts = [p.strip() for p in rest.split(",", 2)]
            # pad to 3 parts
            while len(parts) < 3:
                parts.append("")
            kanji, kana, meaning = parts[0], parts[1], parts[2]
            entries.append((kanji, kana, meaning))
    return entries


def build_pools(
    entries: List[Tuple[str, str, str]],
) -> Tuple[List[str], List[str], List[str]]:
    """
    Build simple pools (unique lists) for kanji, kana and meaning.
    These are used to draw distractors.
    """
    kanji_pool = []
    kana_pool = []
    meaning_pool = []

    seen_kanji = set()
    seen_kana = set()
    seen_meaning = set()

    for kanji, kana, meaning in entries:
        if kanji and kanji not in seen_kanji:
            seen_kanji.add(kanji)
            kanji_pool.append(kanji)
        if kana and kana not in seen_kana:
            seen_kana.add(kana)
            kana_pool.append(kana)
        if meaning and meaning not in seen_meaning:
            seen_meaning.add(meaning)
            meaning_pool.append(meaning)

    return kanji_pool, kana_pool, meaning_pool


# ---- Question generation ----


def generate_all_questions(
    entries: List[Tuple[str, str, str]],
    kanji_pool: List[str],
    kana_pool: List[str],
    meaning_pool: List[str],
    min_options: int = 4,
    rng: Optional[random.Random] = None,
) -> Tuple[List[Dict[str, Any]], int]:
    """
    For each entry produce up to six question dicts (one per requested mapping).
    The six types generated (when possible) are:
      1) kanji -> hiragana
      2) kanji -> meaning
      3) hiragana -> meaning
      4) hiragana -> kanji
      5) meaning -> kanji
      6) meaning -> hiragana

    Returns (questions, skipped_count). A question is produced only if both the
    prompt and the expected correct value exist and at least (min_options - 1)
    distractors can be found from the appropriate pool.
    """
    if rng is None:
        rng = random

    questions: List[Dict[str, Any]] = []
    skipped = 0

    # Prepare unique pools excluding empty values
    kanji_pool_unique = [k for k in dict.fromkeys(kanji_pool) if k]
    kana_pool_unique = [k for k in dict.fromkeys(kana_pool) if k]
    meaning_pool_unique = [m for m in dict.fromkeys(meaning_pool) if m]

    def sample_distractors(
        pool: List[str], correct: str, need: int = 3
    ) -> Optional[List[str]]:
        """Return a list of `need` distractors sampled from `pool` excluding `correct`,
        or None if insufficient distractors exist."""
        candidates = [p for p in pool if p != correct]
        if len(candidates) < need:
            return None
        return rng.sample(candidates, need)

    for idx, (kanji, kana, meaning) in enumerate(entries, start=1):
        # normalize values
        kanji_val = (kanji or "").strip()
        kana_val = (kana or "").strip()
        meaning_val = (meaning or "").strip()

        # Helper to append question dict
        def make_question(
            entry_index: int,
            q_type: str,
            prompt: str,
            text: str,
            correct_value: str,
            options: List[str],
            correct_index: int,
        ):
            q = {
                "entry_index": entry_index,
                "q_type": q_type,
                "prompt": prompt,
                "text": text,
                "options": options,
                "correct_answer": correct_value,
                "correct_index": correct_index,
            }
            questions.append(q)

        # 1) kanji -> hiragana
        if kanji_val and kana_val:
            distractors = sample_distractors(kana_pool_unique, kana_val)
            if distractors is None:
                skipped += 1
            else:
                opts = distractors + [kana_val]
                rng.shuffle(opts)
                make_question(
                    idx,
                    "kanji_to_hiragana",
                    kanji_val,
                    f"What is the hiragana reading of '{kanji_val}'?",
                    kana_val,
                    opts,
                    opts.index(kana_val),
                )

        # 2) kanji -> meaning
        if kanji_val and meaning_val:
            distractors = sample_distractors(meaning_pool_unique, meaning_val)
            if distractors is None:
                skipped += 1
            else:
                opts = distractors + [meaning_val]
                rng.shuffle(opts)
                make_question(
                    idx,
                    "kanji_to_meaning",
                    kanji_val,
                    f"What does '{kanji_val}' mean?",
                    meaning_val,
                    opts,
                    opts.index(meaning_val),
                )

        # 3) hiragana -> meaning
        if kana_val and meaning_val:
            distractors = sample_distractors(meaning_pool_unique, meaning_val)
            if distractors is None:
                skipped += 1
            else:
                opts = distractors + [meaning_val]
                rng.shuffle(opts)
                make_question(
                    idx,
                    "kana_to_meaning",
                    kana_val,
                    f"What does '{kana_val}' mean?",
                    meaning_val,
                    opts,
                    opts.index(meaning_val),
                )

        # 4) hiragana -> kanji
        if kana_val and kanji_val:
            distractors = sample_distractors(kanji_pool_unique, kanji_val)
            if distractors is None:
                skipped += 1
            else:
                opts = distractors + [kanji_val]
                rng.shuffle(opts)
                make_question(
                    idx,
                    "kana_to_kanji",
                    kana_val,
                    f"Which kanji corresponds to '{kana_val}'?",
                    kanji_val,
                    opts,
                    opts.index(kanji_val),
                )

        # 5) meaning -> kanji
        if meaning_val and kanji_val:
            distractors = sample_distractors(kanji_pool_unique, kanji_val)
            if distractors is None:
                skipped += 1
            else:
                opts = distractors + [kanji_val]
                rng.shuffle(opts)
                make_question(
                    idx,
                    "meaning_to_kanji",
                    meaning_val,
                    f"Which kanji represents '{meaning_val}'?",
                    kanji_val,
                    opts,
                    opts.index(kanji_val),
                )

        # 6) meaning -> hiragana
        if meaning_val and kana_val:
            distractors = sample_distractors(kana_pool_unique, kana_val)
            if distractors is None:
                skipped += 1
            else:
                opts = distractors + [kana_val]
                rng.shuffle(opts)
                make_question(
                    idx,
                    "meaning_to_kana",
                    meaning_val,
                    f"What is the hiragana for '{meaning_val}'?",
                    kana_val,
                    opts,
                    opts.index(kana_val),
                )

    return questions, skipped


# ---- Database creation & insert helpers ----


def create_db(db_path: Path, force: bool = False) -> sqlite3.Connection:
    """
    Create (or open) an SQLite DB and ensure the tables exist.
    If `force` is True and the file exists, it will be removed first.
    Returns a sqlite3.Connection.
    """
    db_path = Path(db_path)
    if force and db_path.exists():
        db_path.unlink()

    # Ensure parent dir exists
    if db_path.parent and not db_path.parent.exists():
        db_path.parent.mkdir(parents=True, exist_ok=True)

    conn = sqlite3.connect(str(db_path))
    conn.row_factory = sqlite3.Row

    # Create minimal tables if they do not exist. This keeps compatibility with backend expectations.
    cur = conn.cursor()

    # entries table
    cur.execute(
        """
        CREATE TABLE IF NOT EXISTS entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            list_index INTEGER,
            kanji TEXT,
            kana TEXT,
            meaning TEXT
        );
        """
    )

    # quizzes table (may be used elsewhere)
    cur.execute(
        """
        CREATE TABLE IF NOT EXISTS quizzes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        """
    )

    # questions superset table
    cur.execute(
        """
        CREATE TABLE IF NOT EXISTS questions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            entry_id INTEGER,
            quiz_id INTEGER,
            q_type TEXT,
            prompt TEXT,
            text TEXT,
            options TEXT,
            correct_answer TEXT,
            correct_index INTEGER,
            level INTEGER,
            chapter INTEGER,
            created_at TEXT DEFAULT (datetime('now'))
        );
        """
    )

    # n_level (lightweight)
    cur.execute(
        """
        CREATE TABLE IF NOT EXISTS n_level (
            id INTEGER PRIMARY KEY,
            level TEXT NOT NULL
        );
        """
    )

    # tests table (to store generated tests)
    cur.execute(
        """
        CREATE TABLE IF NOT EXISTS tests (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            questions TEXT,
            created_at TEXT DEFAULT (datetime('now'))
        );
        """
    )

    # indexes for convenience
    cur.execute(
        "CREATE INDEX IF NOT EXISTS idx_entries_list_index ON entries(list_index);"
    )
    cur.execute(
        "CREATE INDEX IF NOT EXISTS idx_questions_entry_id ON questions(entry_id);"
    )
    cur.execute(
        "CREATE INDEX IF NOT EXISTS idx_questions_level_chapter ON questions(level, chapter);"
    )

    conn.commit()
    return conn


def insert_entries(
    conn: sqlite3.Connection, entries: List[Tuple[str, str, str]]
) -> List[int]:
    """
    Insert entries into the `entries` table. Returns list of inserted row ids in same order.
    Each entry is (kanji, kana, meaning). The list_index will be 1-based order.
    """
    cur = conn.cursor()
    inserted_ids: List[int] = []
    for i, (kanji, kana, meaning) in enumerate(entries, start=1):
        cur.execute(
            "INSERT INTO entries (list_index, kanji, kana, meaning) VALUES (?, ?, ?, ?)",
            (i, kanji or None, kana or None, meaning or None),
        )
        inserted_ids.append(cur.lastrowid)
    conn.commit()
    return inserted_ids


def insert_questions(
    conn: sqlite3.Connection, questions: List[Dict[str, Any]], entry_db_ids: List[int]
) -> int:
    """
    Insert generated question dicts into `questions`. `entry_db_ids` is a list returned by insert_entries.
    Returns number of inserted questions.
    """
    cur = conn.cursor()
    inserted = 0
    for q in questions:
        entry_index = q.get("entry_index")
        entry_id = None
        if isinstance(entry_index, int) and 1 <= entry_index <= len(entry_db_ids):
            entry_id = entry_db_ids[entry_index - 1]
        # fallbacks if q contains 'entry_id' already
        if not entry_id and "entry_id" in q:
            entry_id = q.get("entry_id")
        options = q.get("options", []) or []
        options_json = json.dumps(options, ensure_ascii=False)
        correct_answer = q.get("correct_answer")
        correct_index = q.get("correct_index")
        q_type = q.get("q_type")
        prompt = q.get("prompt")
        text = q.get("text")

        cur.execute(
            """
            INSERT INTO questions
            (entry_id, quiz_id, q_type, prompt, text, options, correct_answer, correct_index, level, chapter, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                entry_id,
                None,  # quiz_id (not part of generated questions)
                q_type,
                prompt,
                text,
                options_json,
                correct_answer,
                correct_index,
                q.get("level"),
                q.get("chapter"),
                datetime.utcnow().isoformat(),
            ),
        )
        inserted += 1
    conn.commit()
    return inserted


# ---- If run directly, provide a basic CLI for generating DB from a fixed file ----
if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(
        description="Generate entries and simple MC questions into an SQLite DB from a fixed file."
    )
    parser.add_argument(
        "--fixed",
        "-f",
        type=Path,
        default=Path("mimikara_n3_fixed.txt"),
        help="Fixed input file",
    )
    parser.add_argument(
        "--db",
        "-d",
        type=Path,
        default=Path("mimikara_n3_questions.db"),
        help="Output DB file",
    )
    parser.add_argument("--force", action="store_true", help="Overwrite DB if exists")
    parser.add_argument(
        "--seed", type=int, default=None, help="Random seed for distractors"
    )
    parser.add_argument(
        "--show-sample",
        action="store_true",
        help="Print a few sample questions after generation",
    )

    args = parser.parse_args()

    if args.seed is not None:
        random.seed(args.seed)

    entries = parse_fixed_file(args.fixed)
    print(f"Parsed {len(entries)} entries from {args.fixed}")

    kanji_pool, kana_pool, meaning_pool = build_pools(entries)
    print(
        f"Pools: kanji={len(kanji_pool)}, kana={len(kana_pool)}, meaning={len(meaning_pool)}"
    )

    questions, skipped = generate_all_questions(
        entries, kanji_pool, kana_pool, meaning_pool, rng=random
    )
    print(f"Generated {len(questions)} questions (skipped {skipped})")

    conn = create_db(args.db, force=args.force)
    try:
        ids = insert_entries(conn, entries)
        count = insert_questions(conn, questions, ids)
        print(f"Inserted {len(ids)} entries and {count} questions into {args.db}")
        if args.show_sample:
            cur = conn.cursor()
            cur.execute(
                "SELECT id, entry_id, q_type, prompt, options, correct_index FROM questions ORDER BY id LIMIT 10"
            )
            for row in cur.fetchall():
                opts = json.loads(row[4] or "[]")
                print(f"Q#{row[0]} entry={row[1]} type={row[2]} prompt={row[3]}")
                for i, o in enumerate(opts):
                    mark = "(correct)" if i == row[5] else ""
                    print(f"  {i + 1}. {o} {mark}")
                print()
    finally:
        conn.close()
