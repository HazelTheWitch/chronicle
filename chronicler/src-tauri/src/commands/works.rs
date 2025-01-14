use std::{borrow::Borrow, str::FromStr};

use chronicle::{
    author::AuthorQuery,
    models::{Author, Tag, Work, WorkId},
    record::{Record, RecordDetails},
    search::{Query, QueryTerm},
    tag::{parse::ParsedTag, DiscriminatedTag},
    Chronicle,
};
use tauri::{State, Url};

use crate::create::{AuthorCreate, WorkCreate, WorkEdit};

#[tauri::command]
pub async fn work_query(
    query: String,
    state: State<'_, Chronicle>,
) -> Result<Vec<Work>, chronicle::Error> {
    let mut tx = state.begin().await?;

    let works = if query.is_empty() {
        Work::get_all(&mut tx).await?
    } else {
        Work::search_by_str(&mut tx, &query).await?
    };

    tx.commit().await?;

    Ok(works)
}

#[tauri::command]
pub async fn create_work(
    work_create: WorkCreate,
    state: State<'_, Chronicle>,
) -> Result<WorkId, chronicle::Error> {
    let mut tx = state.begin().await?;

    let author = match &work_create.author {
        Some(create) => create.id.map(AuthorQuery::Id).or(create
            .names
            .first()
            .map(|name| AuthorQuery::Name(name.to_owned()))),
        _ => None,
    };

    let record = Record::from_path(
        state.borrow(),
        work_create.path,
        RecordDetails {
            title: work_create.title,
            url: work_create.url,
            author,
            author_url: None,
            caption: work_create.caption,
            tags: work_create.tags,
        },
    )?;

    let work = Work::create_from_record(&mut tx, &record).await?;

    if let Some(author_create) = work_create.author {
        if let Some(author_id) = work.author_id {
            if let Some(author) = Author::get_by_id(&mut tx, &author_id).await? {
                for name in &author_create.names {
                    author.add_alias(&mut tx, &name).await?;
                }

                for url in author_create
                    .urls
                    .iter()
                    .map(|u| Url::parse(u.as_str()))
                    .collect::<Result<Vec<_>, _>>()?
                {
                    author.add_url(&mut tx, &url).await?;
                }
            }
        }
    }

    tx.commit().await?;

    Ok(work.work_id)
}

#[tauri::command]
pub async fn get_work_edit_by_id(
    id: WorkId,
    state: State<'_, Chronicle>,
) -> Result<WorkEdit, chronicle::Error> {
    let mut tx = state.begin().await?;

    let query = QueryTerm::Id(id).into();

    let mut works = Work::search(&mut tx, &query).await?;

    if works.is_empty() {
        return Err(chronicle::Error::Generic(
            "no work found with that id".to_owned(),
        ));
    };

    let work = works.remove(0);

    let tags = work.get_tags(&mut tx).await?;

    let author = if let Some(author_id) = &work.author_id {
        if let Some(author) = Author::get_by_id(&mut tx, author_id).await? {
            let names = author.get_author_names(&mut tx).await?;
            let urls = author.get_author_urls(&mut tx).await?;

            Some(AuthorCreate {
                urls: urls.into_iter().map(|url| url.url).collect(),
                names: names.into_iter().map(|name| name.name).collect(),
                id: Some(author.author_id),
            })
        } else {
            None
        }
    } else {
        None
    };

    let url = if let Some(url) = work.url {
        Some(url.parse()?)
    } else {
        None
    };

    let work_create = WorkCreate {
        author,
        caption: work.caption,
        path: work.path.into(),
        tags: tags
            .into_iter()
            .map(|tag| DiscriminatedTag {
                name: tag.name,
                discriminator: tag.discriminator,
            })
            .collect(),
        title: work.title,
        url,
    };

    let work_edit = WorkEdit {
        create: work_create,
        work_id: id,
    };

    tx.commit().await?;

    Ok(work_edit)
}

#[tauri::command]
pub async fn import_work_create(
    url: String,
    state: State<'_, Chronicle>,
) -> Result<Vec<WorkCreate>, chronicle::Error> {
    let records = Work::import_records_from_url(&state, &Url::parse(&url)?).await?;

    let mut tx = state.begin().await?;

    let mut works = Vec::with_capacity(records.len());

    for Record {
        path,
        size: _,
        hash: _,
        details,
    } in records
    {
        let mut author = AuthorCreate {
            id: None,
            urls: Vec::new(),
            names: Vec::new(),
        };

        if let Some(query) = details.author {
            let authors = Author::get(&mut tx, &query).await?;

            if authors.len() == 1 {
                author.id = Some(authors[0].author_id);
            }

            match query {
                AuthorQuery::Url(url) => author.urls.push(url.to_string()),
                AuthorQuery::Name(name) => author.names.push(name),
                _ => {}
            }

            if let Some(url) = details.author_url {
                author.urls.push(url.to_string());
            }
        }

        let work = WorkCreate {
            path,
            title: details.title,
            author: Some(author),
            url: details.url,
            caption: details.caption,
            tags: details.tags,
        };

        works.push(work);
    }

    tx.commit().await?;

    Ok(works)
}

#[tauri::command]
pub async fn edit_work(
    work_edit: WorkEdit,
    state: State<'_, Chronicle>,
) -> Result<(), chronicle::Error> {
    let mut tx = state.begin().await?;

    let mut works = Work::search(&mut tx, &QueryTerm::Id(work_edit.work_id).into()).await?;

    if works.is_empty() {
        return Err(chronicle::Error::Generic(String::from(
            "no work found for that id",
        )));
    }

    let mut work = works.remove(0);

    work.url = work_edit.create.url.map(|url| Url::to_string(&url));
    work.title = work_edit.create.title;
    work.caption = work_edit.create.caption;

    work.update(&mut tx).await?;

    let previous_tags = work.get_tags(&mut tx).await?;

    println!("{:?}", previous_tags);

    let discriminated_previous = previous_tags
        .clone()
        .into_iter()
        .map(|t| DiscriminatedTag {
            name: t.name,
            discriminator: t.discriminator,
        })
        .collect::<Vec<_>>();

    for (_, removed) in discriminated_previous
        .iter()
        .zip(previous_tags.iter())
        .filter(|(tag, _)| !work_edit.create.tags.contains(&tag))
    {
        work.remove_tag(&mut tx, removed).await?;
    }

    for added in work_edit
        .create
        .tags
        .iter()
        .filter(|tag| !discriminated_previous.contains(tag))
    {
        let tag = if let Some(tag) =
            Tag::try_get_discriminated(&mut tx, &added.name, added.discriminator.as_deref()).await?
        {
            tag
        } else {
            Tag::create(&mut tx, &added.name, added.discriminator.as_deref()).await?
        };
        work.tag(&mut tx, &tag).await?;
    }

    tx.commit().await?;

    Ok(())
}
