CREATE TABLE "works" (
	"path"	TEXT NOT NULL UNIQUE,
	"work_id"	INTEGER NOT NULL,
    "title" TEXT,
    "url" TEXT,
    "caption" TEXT,
    "author_id" INTEGER,
    FOREIGN KEY("author_id") REFERENCES "authors"("author_id") ON DELETE CASCADE ON UPDATE CASCADE,
	PRIMARY KEY("work_id" AUTOINCREMENT)
);

CREATE TABLE "authors" (
    "name" TEXT NOT NULL UNIQUE,
    "url" TEXT,
    "author_id" INTEGER NOT NULL,
    PRIMARY KEY("author_id" AUTOINCREMENT)
);

CREATE TABLE "tags" (
	"name"	TEXT NOT NULL UNIQUE,
	PRIMARY KEY("name")
);

CREATE TABLE "work_tags" (
	"tag"	TEXT NOT NULL,
	"work_id"	INTEGER NOT NULL,
	FOREIGN KEY("work_id") REFERENCES "works"("work_id") ON DELETE CASCADE ON UPDATE CASCADE,
	FOREIGN KEY("tag") REFERENCES "tags"("name") ON DELETE CASCADE ON UPDATE CASCADE,
	UNIQUE("tag","work_id")
);

CREATE TABLE "meta_tags" (
	"tag"	TEXT NOT NULL,
	"target"	TEXT NOT NULL,
	FOREIGN KEY("tag") REFERENCES "tags"("name") ON DELETE CASCADE ON UPDATE CASCADE,
	FOREIGN KEY("target") REFERENCES "tags"("name") ON DELETE CASCADE ON UPDATE CASCADE,
	UNIQUE("tag","target")
);
