# Chronicle Usage

## Works

### Commands

- `chronicle work add [<details>] <path>`
- `chronicle work import [<details>] <url>`
- `chronicle work search [--read] [<work display options>] <query>`
- `chronicle work list [<work display options>]`

## Tags

### Tag Expressions

Single Tags: `tag1`
Multiple Tags: `tag1,tag2`
Hierarchal Tags: `tag1<tag2`
Grouping Tags: `tag1<(tag2,tag3)`

### Commands

- `chronicle tag work <query> <tags...>`
- `chronicle tag meta <tag statement>`

## Authors

### Author Queries

Name: `author`
Id: `1`
Url: `https://some.artists.url`

### Commands

- `chronicle author list [<author display options>]`
- `chronicle author alias [<author display options>] <author query> <name>`
- `chronicle author add-url [<author display options>] <url>`
