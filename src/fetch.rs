
use {
    crate::models::Feed,
    chrono::prelude::*,
    url::Url,
};

#[derive(Debug, Clone)]
pub struct EpisodeMeta<'a> {
    pub title: Option<&'a str>
}

#[derive(Debug, Clone)]
pub struct Episode<'a> {
    pub meta: EpisodeMeta<'a>,
    pub id:   &'a str,
    pub url:  &'a Url,
    pub when: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct FeedIndex<'a>(Vec<Episode<'a>>);

impl<'a> std::ops::Deref for FeedIndex<'a> {
    type Target = [Episode<'a>];
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'a> From<Vec<Episode<'a>>> for FeedIndex<'a> {
    fn from(mut v: Vec<Episode<'a>>) -> Self {
        // sort the list by descending date
        v.sort_unstable_by_key(|ep| std::cmp::Reverse(ep.when));
        FeedIndex(v)
    }
}

impl<'a> std::iter::FromIterator<Episode<'a>> for FeedIndex<'a> {
    fn from_iter<T>(src: T) -> Self where T: IntoIterator<Item = Episode<'a>> {
        let v: Vec<_> = src.into_iter().collect();
        v.into()
    }
}

#[derive(Debug, Clone)]
pub struct FetchPlan<'a> {
    pub episodes:        &'a [Episode<'a>],
    pub set_fetch_since: Option<DateTime<Utc>>,
}

pub
fn plan_fetch<'a>(feed: &Feed, index: &'a FeedIndex<'a>) -> FetchPlan<'a> {
    // figure out how far back to fetch
    let (threshold, update_db) = {
        // find date of first episode within backlog
        let backlog_start_index = (feed.backlog as usize).max(1).min(index.len()) - 1;
        let backlog_start_date = index[backlog_start_index].when;

        // figure out what date to fetch back to
        if let Some(since) = feed
            .fetch_since
            .map(|naive| DateTime::from_utc(naive, Utc))
            .filter(|since| since <= &backlog_start_date)
        {
            // mature feed - keep fetching from the established date
            (since, false)
        }
        else {
            // new feed, or backlog increased back past since-date - fetch from start of backlog
            (backlog_start_date, true)
        }
    };

    // find the part of the list newer than the threshold
    let split_index = match index.binary_search_by_key(&threshold, |ep| ep.when) {
        Ok(at) => at + 1, // inclusive range
        Err(at) => at
    };

    let episodes = &index[..split_index];
    let set_fetch_since = update_db.then(|| threshold);
    FetchPlan{episodes, set_fetch_since}
}

