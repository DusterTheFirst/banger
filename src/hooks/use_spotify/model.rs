use monostate::MustBe;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Error {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Me {
    /// The name displayed on the user's profile. null if not available.
    pub display_name: Option<String>,
    /// Known external URLs for this user.
    pub external_urls: ExternalUrls,
    /// Information about the followers of the user.
    pub followers: Followers,
    /// A link to the Web API endpoint for this user.
    pub href: String,
    /// The [Spotify user ID][user-id] for the user.
    ///
    /// [user-id]: https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids
    pub id: String,
    /// The user's profile image.
    pub images: Vec<Image>,
    /// The object type: "user".
    #[serde(rename = "type")]
    ty: MustBe!("user"),
    /// The [Spotify URI][s-uri] for the user.
    ///
    /// [s-uri]: https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids
    pub uri: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct ExternalUrls {
    /// The [Spotify URL][s-url] for the object.
    ///
    /// [s-url]: https://developer.spotify.com/documentation/web-api/#spotify-uris-and-ids
    pub spotify: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Followers {
    /// This will always be set to null, as the Web API does not support it at the moment.
    pub href: Option<String>,
    /// The total number of followers.
    pub total: u32,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct Image {
    /// The source URL of the image.
    pub url: String,
    /// The image height in pixels.
    pub height: Option<u32>,
    /// The mage width in pixels.
    pub width: Option<u32>,
}
