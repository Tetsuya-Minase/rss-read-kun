pub struct RssData<'a> {
    pub title:Option<&'a String>,
    pub description: Option<&'a String>,
    pub link: Option<&'a String>,
}
