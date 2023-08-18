use std::{cmp::Ordering, fmt};

use mime::{Mime, Name};

use crate::Quality;

/// A Media Type combines a media range (including parameters) with a quality weight.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaType {
    pub mime: Mime,
    pub quality: Option<Quality>,
    /// The specificity of a media type is the number of parameters, excluding the quality.
    specificity: u8,
}

impl MediaType {
    /// The quality factor for a media type is 1.0 if not explicitly specified with the `q` parameter.
    #[inline]
    pub fn quality_factor(&self) -> Quality {
        self.quality.unwrap_or(Quality::new(1.0))
    }
}

impl Ord for MediaType {
    /// Media types compare by the following rules:
    ///
    /// 1. `quality_factor()`
    /// 2. `specificity`
    /// 3. `Reverse(mime.essence_str())`
    ///     - wildcard subtypes are less than any explicit type
    ///     - wildcard outer types are less than any explicit type
    ///     - we reverse the essence str otherwise to conform to the intuition that the alphabetically lowest value is the first considered
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        /// Compare two mime names, returning an ordering.
        ///
        /// We can't compare them directly because the derived implementation doesn't
        /// respect case insensitivity, and also doesn't handle the "wildcard is least"
        /// rule that we want.
        fn compare_names(a: Name, b: Name) -> Ordering {
            match (a, b) {
                (mime::STAR, mime::STAR) => Ordering::Equal,
                (mime::STAR, _) => Ordering::Less,
                (_, mime::STAR) => Ordering::Greater,
                (a, b) => {
                    // this _may_ be equal even if the source strings are unequal, if it happens that
                    // we have a case-insensitive comparison. Unfortunately, the library doesn't expose
                    // that information to us directly, so we have to explicitly compare for equality before
                    // performing an ordering comparison.
                    //
                    // ... the _only_ way to get at a safe public test which respects case insensitivity when appropriate
                    // is to compare a `Name` to a `&str`.
                    if a == b.as_str() {
                        Ordering::Equal
                    } else {
                        // if they're unequal, by whatever metric, it's fine to fall back to string comparison.
                        // note though that we reverse the output: this lets us sort a list of media types, and the
                        // best one (all else being equal) is the first one alphabetically.
                        a.as_str().cmp(b.as_str()).reverse()
                    }
                }
            }
        }

        self.quality_factor()
            .cmp(&other.quality_factor())
            .then_with(|| self.specificity.cmp(&other.specificity))
            .then_with(|| {
                // if the outer types are identical, what matters are the subtypes
                // don't forget that this might need to respect case insensitivity!
                if self.mime.type_() == other.mime.type_().as_str() {
                    compare_names(self.mime.subtype(), other.mime.subtype())
                } else {
                    compare_names(self.mime.type_(), other.mime.type_())
                }
            })
    }
}

impl PartialOrd for MediaType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl From<Mime> for MediaType {
    fn from(mime: Mime) -> Self {
        let quality = mime
            .get_param("q")
            .map(|name| name.as_str())
            .and_then(|s| s.parse::<f32>().ok())
            .map(|f| f.clamp(0.0, 1.0))
            .and_then(Quality::try_new);

        // technically, this is a bug: if two mime types have more than 255 parameters, they might compare as equal when they are not.
        // it is an unlikely enough scenario that I do not intend to fix it.
        let specificity = mime
            .params()
            .filter(|(key, _value)| key != &"q")
            .count()
            .try_into()
            .unwrap_or(u8::MAX);

        MediaType {
            mime,
            quality,
            specificity,
        }
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { mime, .. } = self;
        write!(f, "{mime}")
    }
}
