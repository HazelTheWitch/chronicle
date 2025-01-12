ALTER TABLE
    "tags" RENAME TO "old_tags";

CREATE TABLE "tags" (
    "name" TEXT NOT NULL COLLATE NOCASE,
    "id" INTEGER NOT NULL UNIQUE,
    "discriminator" TEXT COLLATE NOCASE,
    UNIQUE("name", "discriminator"),
    PRIMARY KEY("id" AUTOINCREMENT)
);

INSERT INTO
    "tags"("name", "id")
SELECT
    *
FROM
    "old_tags";

DROP TABLE "old_tags";

ALTER TABLE
    "tags" RENAME TO "old_tags";

ALTER TABLE
    "old_tags" RENAME TO "tags";

CREATE TRIGGER "tag_uniqueness_with_discriminator" BEFORE
INSERT
    ON "tags" FOR EACH ROW
BEGIN
SELECT
    RAISE(ABORT, "Non Unique Tag Detected")
WHERE
    EXISTS (
        SELECT
            1
        FROM
            "tags"
        WHERE
            "tags"."name" = NEW."name"
            AND (
                (
                    NEW."discriminator" IS NULL
                    AND "tags".discriminator IS NOT NULL
                )
                OR (
                    NEW."discriminator" IS NOT NULL
                    AND "tags"."discriminator" IS NULL
                )
            )
    );

END;