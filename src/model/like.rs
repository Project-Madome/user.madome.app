use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use hyper::{header, Body, Request, Response, StatusCode};
use madome_sdk::api::{header::take_origin_response, library, Token};
use serde::{Deserialize, Serialize};
use util::http::SetResponse;
use uuid::Uuid;

use crate::{config::Config, entity};

use super::Presenter;

/* #[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")] */
pub enum Like {
    Book {
        book_id: u32,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        user_id: Uuid,
        created_at: DateTime<Utc>,
    },
}

impl Like {
    pub fn book_id(&self) -> Option<u32> {
        match self {
            Like::Book { book_id, .. } => Some(*book_id),
            _ => None,
        }
    }

    pub fn book_tag(&self) -> Option<(&str, &str)> {
        match self {
            Like::BookTag {
                tag_kind, tag_name, ..
            } => Some((tag_kind as &str, tag_name as &str)),
            _ => None,
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ReducedLike {
    Book {
        book_id: u32,
        created_at: DateTime<Utc>,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        created_at: DateTime<Utc>,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtendedLike {
    Book {
        book_id: u32,
        created_at: DateTime<Utc>,
        book: library::model::Book,
    },
    BookTag {
        tag_kind: String,
        tag_name: String,
        created_at: DateTime<Utc>,
        books: Vec<library::model::Book>,
    },
}

impl ExtendedLike {
    /*     async fn from_like(
        like: Like,
        token: impl Into<Token<'_>>,
        library_url: impl Into<String>,
    ) -> crate::Result<Self> {
        match like {
            Like::Book {
                book_id,
                created_at,
                ..
            } => {
                let book = library::get_book_by_id(library_url, token, book_id).await?;

                Ok(Self::Book {
                    book_id,
                    created_at,
                    book,
                })
            }
            Like::BookTag {
                tag_kind,
                tag_name,
                created_at,
                ..
            } => {
                let books = library::get_books_by_tags(
                    library_url,
                    token,
                    tags,
                    // TODO: per_page ?????? ??????
                    3,
                    1,
                    None,
                )
                .await?;

                Ok(Self::BookTag {
                    tag_kind,
                    tag_name,
                    books,
                    created_at,
                })
            }
        }
    } */
}

#[async_trait::async_trait]
impl Presenter for Vec<Like> {
    async fn set_response(
        self,
        request: &mut Request<Body>,
        resp: &mut Response<Body>,
        config: Arc<Config>,
    ) -> crate::Result<()> {
        let serialized = match take_origin_response(request.headers()) {
            // for internal
            true => {
                let likes: Vec<ReducedLike> = self.into_iter().map(Into::into).collect();
                serde_json::to_vec(&likes).expect("json serialize")
            }
            // for external
            false => {
                let book_tags = self
                    .iter()
                    .filter_map(|x| x.book_tag())
                    .map(|(x, y)| (x.to_owned(), y.to_owned()))
                    .collect::<Vec<_>>();

                let book_ids = self.iter().filter_map(|x| x.book_id()).collect::<Vec<_>>();

                let (books, mut books_group_by_tags) = futures::try_join!(
                    async {
                        if book_ids.is_empty() {
                            Ok(Vec::new())
                        } else {
                            library::get_books_by_ids(
                                config.library_url(),
                                Token::default(),
                                book_ids,
                            )
                            .await
                        }
                    },
                    async {
                        if book_tags.is_empty() {
                            Ok(HashMap::new())
                        } else {
                            #[derive(Debug, Default, Deserialize)]
                            struct Payload {
                                #[serde(rename = "books-per-page")]
                                per_page: Option<usize>,
                                #[serde(rename = "books-page")]
                                page: Option<usize>,
                                #[serde(rename = "books-sort-by")]
                                sort_by: Option<String>,
                            }

                            let qs = request.uri().query().unwrap_or_default();
                            let Payload {
                                per_page,
                                page,
                                sort_by,
                            } = serde_qs::from_str(qs).unwrap_or_default();

                            let sort_by = match sort_by.as_deref() {
                                Some("id-desc") => Some(library::payload::BookSortBy::IdDesc),
                                Some("id-asc") => Some(library::payload::BookSortBy::IdAsc),
                                Some("random") => Some(library::payload::BookSortBy::Random),
                                _ => None,
                            };

                            library::get_books_by_tags(
                                config.library_url(),
                                Token::default(),
                                book_tags,
                                per_page.unwrap_or(3),
                                page.unwrap_or(1),
                                sort_by,
                            )
                            .await
                        }
                    }
                )?;
                let mut books = books
                    .into_iter()
                    .map(|x| (x.id, x))
                    .collect::<HashMap<_, _>>();

                let likes = self
                    .into_iter()
                    // Library?????? ????????? ??? ?????? ????????? ????????????
                    .filter_map(|x| match x {
                        Like::Book {
                            book_id,
                            created_at,
                            ..
                        } => {
                            let book = books.remove(&book_id);

                            book.map(|x| ExtendedLike::Book {
                                book_id,
                                created_at,
                                book: x,
                            })

                            /* match book {
                                Some(book) => Some(ExtendedLike::Book {
                                    book_id,
                                    created_at,
                                    book,
                                }),
                                None => {
                                    // TODO: LIKE??? ????????? ??? ?????? ???????????? ????????? ?????????
                                    // ????????? ?????? ????????? ??? ??? ?????? ???????????? filter_map()?????? ????????? ???????????????
                                    // panic!("why hasn't book {book_id}");
                                    None
                                }
                            } */
                        }
                        Like::BookTag {
                            tag_kind,
                            tag_name,
                            created_at,
                            ..
                        } => {
                            let tag = (tag_kind, tag_name);

                            // log::debug!("books = {books_group_by_tags:#?}");

                            // TODO: LIKE??? ????????? ??? ?????? ???????????? ????????? ????????? ?????? ??? ???????????? panic!??? ???????????? ??? ??????????
                            // ????????? panic!??? ???????????? ?????? ????????? ??????????????? ??? ?????????
                            // ?????? ????????? ????????? ???????????? ???????????? ??? ????????? ?????? ??? ????????? ???
                            let books = books_group_by_tags.remove(&tag); // .unwrap_or_default(); //.expect("why hasn't book?");

                            books.map(|xs| ExtendedLike::BookTag {
                                tag_kind: tag.0,
                                tag_name: tag.1,
                                books: xs,
                                created_at,
                            })
                        }
                    })
                    .collect::<Vec<_>>();

                serde_json::to_vec(&likes).expect("json serialize")
                // library

                // tag_type = if female || male -> tag
                /*
                let cookie = Cookie::from(resp.headers());
                let token: Token = cookie
                    .get2(MADOME_ACCESS_TOKEN, MADOME_REFRESH_TOKEN)
                    .unwrap_or_default()
                    .into();

                let book_ids = self.iter().filter_map(Like::book_id).collect::<Vec<_>>();
                let books =
                    library::get_books_by_ids(config.library_url(), token.clone(), book_ids)
                        .await?;

                // TODO: ????????? library ????????? ???????????? ??????
                fn to_old_tag((kind, name): (&str, &str)) -> (String, String) {
                    let metadata_value = match kind {
                        // kind = female, name = loli
                        x @ "female" | x @ "male" => format!("{x} {name}"),
                        _ => name.to_string(),
                    };
                    let metadata_type = match kind {
                        "artist" | "series" | "group" | "character" => kind,
                        // "female" | "male" => "tag",
                        _ => "tag",
                    }
                    .to_string();

                    (metadata_type, metadata_value)
                }

                fn to_new_tag(
                    (metadata_type, metadata_value): (String, String),
                ) -> (String, String) {
                    let name = match metadata_type {};
                }

                let r = self
                    .iter()
                    .filter_map(Like::book_tag)
                    // TODO: ????????? library ????????? ???????????? ??????
                    .map(to_old_tag)
                    .map(|(kind, name)| async {
                        let r = library::get_books_by_metadata(
                            config.library_url(),
                            token.clone(),
                            None,
                            &kind,
                            &name,
                            3,
                            1,
                            None,
                        )
                        .await?;

                        Ok(((kind, name), r))
                    })
                    .task(10)
                    .try_collect::<Vec<((String, String), Vec<library::model::Book>)>>()
                    .await?;

                serde_json::to_vec(&books).expect("json serialize") */
            }
        };

        resp.set_status(StatusCode::OK).unwrap();
        resp.set_header(header::CONTENT_TYPE, "application/json")
            .unwrap();
        resp.set_body(serialized.into());

        Ok(())
    }
}

/* impl Presenter for Like {
    fn to_response(self, response: hyper::http::response::Builder) -> hyper::Response<hyper::Body> {
        let like: LikeWithoutUserId = self.into();
        let serialized = serde_json::to_string(&like).expect("json serialize");

        response
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(serialized.into())
            .unwrap()
    }
} */

impl From<Like> for ReducedLike {
    fn from(like: Like) -> Self {
        match like {
            Like::Book {
                book_id,
                created_at,
                ..
            } => Self::Book {
                book_id,
                created_at,
            },
            Like::BookTag {
                tag_kind,
                tag_name,
                created_at,
                ..
            } => Self::BookTag {
                tag_kind,
                tag_name,
                created_at,
            },
        }
    }
}

impl From<entity::Like> for Like {
    fn from(like: entity::Like) -> Self {
        match like {
            entity::Like::Book {
                book_id,
                user_id,
                created_at,
            } => Self::Book {
                book_id,
                user_id,
                created_at,
            },
            entity::Like::BookTag {
                tag_kind,
                tag_name,
                user_id,
                created_at,
            } => Self::BookTag {
                tag_kind,
                tag_name,
                user_id,
                created_at,
            },
        }
    }
}
