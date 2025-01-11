CREATE TABLE "works" (
    "path" TEXT NOT NULL UNIQUE,
    "work_id" INTEGER NOT NULL,
    "size" INTEGER NOT NULL,
    "title" TEXT,
    "url" TEXT,
    "caption" TEXT,
    "author_id" INTEGER,
    "hash" INTEGER NOT NULL UNIQUE,
    FOREIGN KEY("author_id") REFERENCES "authors"("author_id") ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY("work_id" AUTOINCREMENT)
);

CREATE TABLE "authors" (
    "name" TEXT NOT NULL UNIQUE COLLATE NOCASE,
    "author_id" INTEGER NOT NULL,
    PRIMARY KEY("author_id" AUTOINCREMENT)
);

CREATE TABLE "author_urls" (
    "author_id" INTEGER NOT NULL,
    "url" TEXT NOT NULL,
    FOREIGN KEY("author_id") REFERENCES "authors"("author_id") ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE("url")
);

CREATE TABLE "tags" (
    "name" TEXT NOT NULL UNIQUE COLLATE NOCASE,
    "id" INTEGER NOT NULL UNIQUE,
    PRIMARY KEY("id" AUTOINCREMENT)
);

CREATE TABLE "work_tags" (
    "tag" INTEGER NOT NULL,
    "work_id" INTEGER NOT NULL,
    FOREIGN KEY("work_id") REFERENCES "works"("work_id") ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY("tag") REFERENCES "tags"("id") ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE("tag", "work_id"),
    PRIMARY KEY ("tag", "work_id")
);

CREATE TABLE "meta_tags" (
    "tag" INTEGER NOT NULL,
    "target" INTEGER NOT NULL,
    FOREIGN KEY("tag") REFERENCES "tags"("id") ON DELETE CASCADE ON UPDATE CASCADE,
    FOREIGN KEY("target") REFERENCES "tags"("id") ON DELETE CASCADE ON UPDATE CASCADE,
    UNIQUE("tag", "target"),
    PRIMARY KEY ("tag", "target")
);

CREATE TRIGGER "cycle_check" BEFORE
INSERT
    ON "meta_tags" FOR EACH ROW
BEGIN
SELECT
    RAISE(ABORT, "Tag Cycle Detected")
WHERE
    EXISTS (
        WITH RECURSIVE "w"(
            "parent",
            "last_visited",
            "already_visited",
            "cycle"
        ) AS (
            SELECT
                DISTINCT "target" AS "parent",
                "tag" AS "last_visited",
                "target" AS "already_visited",
                0 AS "cycle"
            FROM
                "meta_tags"
            UNION
            ALL
            SELECT
                "t"."target" AS "parent",
                "t"."tag" AS "last_visited",
                "already_visited" || ',' || "t"."target",
                "already_visited" LIKE '%' || "t"."target" || '%'
            FROM
                "meta_tags" AS "t"
                JOIN "w" ON "w"."last_visited" = "t"."target"
            WHERE
                NOT "cycle"
        )
        SELECT
            1
        FROM
            "w"
        WHERE
            "last_visited" = NEW."target"
            AND "already_visited" LIKE '%' || NEW."tag" || '%'
    );

END;
