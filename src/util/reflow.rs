use super::*;

/// Encapsulate reflow logic for buffer manipulation.
pub struct Reflow<'a> {
    buf: &'a mut Buffer,
    range: Range,
    text: String,
    limit: usize,
}

impl<'a> Reflow<'a> {
    /// Create a reflow instance, where buffer and range determine the target,
    /// and the limit is the maximum length of a line, regardless of prefixes.
    pub fn new(
        buf: &'a mut Buffer, range: Range, limit: usize
    ) -> std::result::Result<Self, Error> {
        let text = buf.read(&range).ok_or("Selection is invalid.")?;
        Ok(Self { buf, range, text, limit })
    }

    pub fn apply(mut self) -> std::result::Result<(), Error> {
        let prefix = self.infer_prefix()?;
        let jtxt = self.justify_str(&prefix);
        self.buf.delete_range(self.range.clone());
        self.buf.cursor.move_to(self.range.start());
        self.buf.insert(jtxt);

        Ok(())
    }

    fn infer_prefix(&self) -> std::result::Result<String, Error> {
        match self.text.split_whitespace().next() {
        	Some(n) => if n.chars().next().unwrap().is_alphanumeric() {
        	    Ok("".to_string())
        	} else {
        	    Ok(n.to_string())
        	},
        	None => bail!("Selection is empty."),
        }
    }


    fn justify_str(&mut self, prefix: &str) -> String {
        let text = self.buf.read(&self.range).unwrap();
        let mut limit = self.limit;
        let mut justified = String::with_capacity(text.len());
        let mut pars = text.split("\n\n").peekable();

        let mut space_delims = ["".to_string(), " ".to_string(), "\n".to_string()];
        if prefix != "" {
        	space_delims[0] += prefix;
        	space_delims[0] += " ";
        	space_delims[2] += prefix;
        	space_delims[2] += " ";
        	limit -= prefix.len() + 1;
        }

        while let Some(par) = pars.next() {
        	let mut words = par.split_whitespace();
        	let mut len = 0;
        	let mut first = true;

        	while let Some(word) = words.next() {
        	    if word == prefix {
        		continue;
        	    }

        	    len += word.len();

        	    let over = len > limit;
        	    let u_over = over as usize;
        	    let idx = (!first as usize) * u_over + !first as usize;

        	    justified += &space_delims[idx];
        	    justified += word;

        	    // if we're over, set the length to 0, otherwise increment it
        	    // properly. This just does that mith multiplication by 0 instead of
        	    // branching.
        	    len = (len + 1) * (1 - u_over) + (word.len() + 1) * u_over;
        	    first = false;
        	}

        	if pars.peek().is_some() {
        	    justified += "\n\n"; // add back the paragraph break.
        	}
        }

        justified
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // as simple as it gets: one character words for easy debugging.
    #[test]
    fn justify_simple() {
        let mut buf = Buffer::new();
        buf.insert("\
a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a\n");

        Reflow::new(
            &mut buf,
            Range::new(
                scribe::buffer::Position { line: 0, offset: 0 },
                scribe::buffer::Position { line: 1, offset: 0 },
            ),
            80,
        ).unwrap().apply().unwrap();

        assert_eq!(
            buf.data(),
	        "\
a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a
a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a"
    	);
    }

    #[test]
    fn justify_paragraph() {
        let mut buf = Buffer::new();
    	buf.insert("\
these are words to be used as demos for the thing that this is. this is text \
reflowing and justification over a few lines. this is just filler text in case \
it wasn't obvious.\n"
        );

        Reflow::new(
            &mut buf,
            Range::new(
                scribe::buffer::Position { line: 0, offset: 0 },
                scribe::buffer::Position { line: 1, offset: 0 },
            ),
            80,
        ).unwrap().apply().unwrap();
    	assert_eq!(
    	    buf.data(), "\
these are words to be used as demos for the thing that this is. this is text
reflowing and justification over a few lines. this is just filler text in case
it wasn't obvious."
       	);
    }

    #[test]
    fn justify_multiple_pars() {
        let mut buf = Buffer::new();
    	buf.insert("\
Here's more filler text! So fun fact of the day, I was trying to just copy paste \
some lorem ipsum to annoy my latin student friends, but honestly it broke the \
M-q 'justify' function in emacs, which makes it a bit difficult to work with. \
Overall, it's just not that great with code!

Fun fact of the day number two, writing random paragraphs of text is honestly \
taking way more effort than I anticipated, and I deeply apologize for the lack \
of sanity and coherence here!

Fun fact of the day number three is that I spent three hours getting this to not \
branch. There is no way that that micro-optimization will actually save three \
hours worth of time, but I did it anyway for no good reason!\n"
        );

        Reflow::new(
            &mut buf,
            Range::new(
                scribe::buffer::Position { line: 0, offset: 0 },
                scribe::buffer::Position { line: 5, offset: 0 },
            ),
            80,
        ).unwrap().apply().unwrap();

    	assert_eq!(
    	    buf.data(), "\
Here's more filler text! So fun fact of the day, I was trying to just copy paste
some lorem ipsum to annoy my latin student friends, but honestly it broke the
M-q 'justify' function in emacs, which makes it a bit difficult to work with.
Overall, it's just not that great with code!

Fun fact of the day number two, writing random paragraphs of text is honestly
taking way more effort than I anticipated, and I deeply apologize for the lack
of sanity and coherence here!

Fun fact of the day number three is that I spent three hours getting this to not
branch. There is no way that that micro-optimization will actually save three
hours worth of time, but I did it anyway for no good reason!"
    	);
    }

    #[test]
    fn justify_simple_prefix() {
        let mut buf = Buffer::new();
    	buf.insert("\
# a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a\n"
        );
        Reflow::new(
            &mut buf,
            Range::new(
                scribe::buffer::Position { line: 0, offset: 0 },
                scribe::buffer::Position { line: 1, offset: 0 },
            ),
            80,
        ).unwrap().apply().unwrap();

    	assert_eq!(
    	    buf.data(), "\
# a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a
# a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a a"
    	);
    }

    #[test]
    fn justify_paragraph_prefix() {
        let mut buf = Buffer::new();
        buf.insert("\
// filler text meant
// to do stuff and things that  end up with text nicely \
wrappped around a comment delimiter such as the double slashes in c-style \
languages.\n"
        );

        Reflow::new(
            &mut buf,
            Range::new(
                scribe::buffer::Position { line: 0, offset: 0 },
                scribe::buffer::Position { line: 2, offset: 0 },
            ),
            80,
        ).unwrap().apply().unwrap();

    	assert_eq!(
    	    buf.data(), "\
// filler text meant to do stuff and things that end up with text nicely
// wrappped around a comment delimiter such as the double slashes in c-style
// languages.",
    	);
    }
}
