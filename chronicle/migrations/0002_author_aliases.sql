ALTER TABLE
    "authors" RENAME TO "old_authors";

CREATE TABLE "authors" (
    "author_id" INTEGER NOT NULL,
    PRIMARY KEY("author_id" AUTOINCREMENT)
);

CREATE TABLE "author_names" (
    "author_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL COLLATE NOCASE,
    FOREIGN KEY("author_id") REFERENCES "authors"("author_id") ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE ("author_id", "name"),
    PRIMARY KEY ("author_id", "name")
);

INSERT INTO
    "authors" ("author_id")
SELECT
    "author_id"
FROM
    "old_authors";

INSERT INTO
    "author_names" ("author_id", "name")
SELECT
    "author_id",
    "name"
FROM
    "old_authors";

DROP TABLE "old_authors";

ALTER TABLE
    "authors" RENAME TO "old_authors";

ALTER TABLE
    "old_authors" RENAME TO "authors";