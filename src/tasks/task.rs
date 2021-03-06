#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    inner: ::todo_txt::Task,
    pub id: usize,
    pub note: super::Note,
    pub recurrence: Option<super::Recurrence>,
    pub flagged: bool,
}

impl Task
{
    pub fn new() -> Self
    {
        Self {
            inner: Default::default(),
            id: 0,
            note: super::Note::None,
            recurrence: None,
            flagged: false,
        }
    }

    pub fn markup_subject(&self) -> String
    {
        let mut subject = self.subject.clone();

        let regex = ::regex::Regex::new("(?P<url>[\\w]+://[^\\s]+)")
            .unwrap();
        subject = regex.replace_all(&subject, |caps: &::regex::Captures| {
            format!("<a href=\"{url}\">{url}</a>", url=caps[1].replace("&", "&amp;"))
        }).into_owned();

        let regex = ::regex::Regex::new("(?P<space>^|[\\s])(?P<tag>[\\+@][\\w-]+)")
            .unwrap();
        subject = regex.replace_all(&subject, "$space<b>$tag</b>")
            .into_owned();

        subject
    }

    fn note(task: &::todo_txt::Task) -> super::Note
    {
        let tag = match ::std::env::var("TODO_NOTE_TAG") {
            Ok(tag) => tag,
            Err(_) => "note".to_owned(),
        };

        if let Some(file) = task.tags.get(&tag) {
            super::Note::from_file(file)
        }
        else {
            super::Note::None
        }
    }

    pub fn complete(&mut self)
    {
        let today = ::date::today();

        self.finished = true;
        self.finish_date = Some(today);
    }

    pub fn uncomplete(&mut self)
    {
        self.finished = false;
        self.finish_date = None;
    }
}

impl ::std::str::FromStr for Task
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()>
    {
        let mut task = ::todo_txt::Task::from_str(s)?;

        let note = Self::note(&task);
        task.tags.remove(&"note".to_owned());

        let mut recurrence = None;

        if let Some(rec) = task.tags.get(&"rec".to_owned()) {
            recurrence = match super::Recurrence::from_str(rec) {
                Ok(rec) => Some(rec),
                Err(_) => None,
            };
        }
        task.tags.remove(&"rec".to_owned());

        let flagged = task.tags.contains_key(&"f".to_owned());
        task.tags.remove(&"f".to_owned());

        Ok(Self {
            id: 0,
            note,
            inner: task,
            recurrence,
            flagged,
        })
    }
}

impl ::std::ops::Deref for Task
{
    type Target = ::todo_txt::Task;

    fn deref(&self) -> &Self::Target
    {
        &self.inner
    }
}

impl ::std::ops::DerefMut for Task
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        &mut self.inner
    }
}

impl ::std::fmt::Display for Task
{
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result
    {
        use std::ops::Deref;

        f.write_str(format!("{}", self.deref()).as_str())?;

        if self.note != ::tasks::Note::None {
            f.write_str(format!(" {}", self.note).as_str())?;
        }

        if let Some(ref recurrence) = self.recurrence {
            f.write_str(format!(" rec:{}", recurrence).as_str())?;
        }

        if self.flagged {
            f.write_str(" f:1")?;
        }

        Ok(())
    }
}

impl ::std::cmp::PartialOrd for Task
{
    fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering>
    {
        Some(self.cmp(other))
    }
}

impl ::std::cmp::Ord for Task
{
    fn cmp(&self, other: &Self) -> ::std::cmp::Ordering
    {
        if self.inner.due_date != other.inner.due_date {
            return self.inner.due_date.cmp(&other.inner.due_date);
        }

        if self.inner.priority != other.inner.priority {
            return self.inner.priority.cmp(&other.inner.priority)
                .reverse();
        }

        if self.inner.subject != other.inner.subject {
            return self.inner.subject.cmp(&other.inner.subject);
        }

        ::std::cmp::Ordering::Equal
    }
}
