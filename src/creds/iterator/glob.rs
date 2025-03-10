use crate::{creds, session::Error};

pub(crate) struct Glob {
    paths: glob::Paths,
    elements: usize,
}

impl Glob {
    pub fn new(pattern: String) -> Result<Self, Error> {
        // validate the pattern and count the elements first
        let paths = match glob::glob(&pattern) {
            Err(e) => return Err(e.to_string()),
            Ok(paths) => paths,
        };
        let elements = paths.count();
        let paths = glob::glob(&pattern).unwrap();

        Ok(Self { paths, elements })
    }
}

impl creds::Iterator for Glob {
    fn search_space_size(&self) -> usize {
        self.elements
    }
}

impl std::iter::Iterator for Glob {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.paths.next() {
            if let Ok(path) = next {
                return Some(path.to_str().unwrap().to_owned());
            } else {
                log::error!("glob error: {:?}", next);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use crate::creds::{iterator, Expression};

    #[test]
    fn can_handle_glob() {
        let num_files = 3;
        let tmpdir = tempfile::tempdir().unwrap();
        let tmpdirname = tmpdir.path().to_str().unwrap().to_owned();
        let mut expected = vec![];

        for i in 0..num_files {
            let filename = format!("test{}.txt", i);
            let tmppath = tmpdir.path().join(&filename);
            let mut tmpfile = File::create(&tmppath).unwrap();

            write!(tmpfile, "test\n").unwrap();
            tmpfile.flush().unwrap();
            drop(tmpfile);

            expected.push(format!("{}/{}", &tmpdirname, filename));
        }

        let gen = iterator::new(Expression::Glob {
            pattern: format!("{}/*.txt", tmpdirname),
        })
        .unwrap();
        let tot = gen.search_space_size();
        let vec: Vec<String> = gen.collect();

        assert_eq!(tot, expected.len());
        assert_eq!(vec, expected);
    }
}
