//! Built-in sentence bank for typing tests.

use rand::seq::IndexedRandom;

/// Categories for sentence length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentenceLength {
    Short,
    Medium,
    Long,
}

impl std::str::FromStr for SentenceLength {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "short" => Ok(SentenceLength::Short),
            "medium" => Ok(SentenceLength::Medium),
            "long" => Ok(SentenceLength::Long),
            _ => Err("invalid length; must be short, medium, or long"),
        }
    }
}

/// ~20 words
const SHORT_SENTENCES: &[&str] = &[
    "The quick brown fox jumps over the lazy dog in the middle of a sunny afternoon.",
    "A journey of a thousand miles begins with a single step, no matter how hard it seems.",
    "To be or not to be, that is the question that has puzzled philosophers for many centuries.",
    "She sells seashells by the seashore, but the shells she sells are surely not seashells.",
    "All that glitters is not gold, often have you heard that told; many a man his life hath sold.",
    "How much wood would a woodchuck chuck if a woodchuck could really chuck any wood at all?",
    "Programming is the art of telling another human what one wants the computer to actually do.",
    "There is no place like home, especially when you have traveled far and wide across the earth.",
    "The early bird catches the worm, but the second mouse gets the cheese in the trap.",
    "Actions speak louder than words, but sometimes silence can be the most deafening sound of all."
];

/// ~40 words
const MEDIUM_SENTENCES: &[&str] = &[
    "In the beginning, the universe was created. This has made a lot of people very angry and been widely regarded as a bad move. Many races believe that it was created by some sort of god.",
    "It is a truth universally acknowledged, that a single man in possession of a good fortune, must be in want of a wife. However little known the feelings or views of such a man may be.",
    "Call me Ishmael. Some years ago, never mind how long precisely, having little or no money in my purse, and nothing particular to interest me on shore, I thought I would sail about a little.",
    "The sun did not shine. It was too wet to play. So we sat in the house all that cold, cold, wet day. I sat there with Sally. We sat there, we two. And I said, 'How I wish we had something to do!'",
    "It was the best of times, it was the worst of times, it was the age of wisdom, it was the age of foolishness, it was the epoch of belief, it was the epoch of incredulity.",
    "I am invisible, understand, simply because people refuse to see me. Like the bodiless heads you see sometimes in circus sideshows, it is as though I have been surrounded by mirrors of hard, distorting glass.",
    "Whether I shall turn out to be the hero of my own life, or whether that station will be held by anybody else, these pages must show. To begin my life with the beginning of my life.",
    "There was no possibility of taking a walk that day. We had been wandering, indeed, in the leafless shrubbery an hour in the morning; but since dinner the cold winter wind had brought with it clouds so sombre.",
    "Mr. and Mrs. Dursley, of number four, Privet Drive, were proud to say that they were perfectly normal, thank you very much. They were the last people you'd expect to be involved in anything strange or mysterious.",
    "Far out in the uncharted backwaters of the unfashionable end of the western spiral arm of the Galaxy lies a small unregarded yellow sun. Orbiting this at a distance of roughly ninety-two million miles is an utterly insignificant little blue green planet."
];

/// ~60+ words
const LONG_SENTENCES: &[&str] = &[
    "The rustling of the leaves in the gentle autumn breeze brought a sense of peace to the weary traveler, who had spent the last several days traversing the treacherous mountain pass in search of the fabled hidden city that was said to contain untold treasures and ancient knowledge beyond mortal comprehension, only to find a quiet valley filled with nothing but old memories.",
    "As the thunderstorm raged outside, casting eerie shadows across the dimly lit room with each flash of lightning, the detective carefully examined the cryptic note left at the scene of the crime, searching for any clue that might reveal the identity of the elusive thief who had managed to bypass the state-of-the-art security system without leaving a single trace behind, confounding the entire police force.",
    "In a world where technology has advanced to the point of seamlessly integrating with human consciousness, the ethical implications of artificial intelligence and cybernetic enhancements continue to be a topic of fierce debate among scholars, politicians, and the general public, as society struggles to define the boundaries between humanity and machines in an ever-evolving landscape of rapid technological progress and innovation.",
    "The intricate mechanism of the antique pocket watch fascinated the young horologist, who spent countless hours meticulously disassembling and reassembling its delicate gears, springs, and escapement, marveling at the craftsmanship and ingenuity of the master watchmaker who had created such a precise and beautiful timepiece over a century ago, a testament to an era before the advent of digital technology.",
    "Navigating the complex labyrinth of bureaucratic red tape required to secure the necessary permits for the construction of the new community center proved to be a daunting task for the local neighborhood association, testing their patience and resolve as they attended endless council meetings, filled out mountains of paperwork, and negotiated with various city officials to finally realize their dream of a shared space.",
    "The culinary arts demand not only a deep understanding of flavor profiles, cooking techniques, and ingredient combinations, but also a passion for creativity and presentation, transforming a simple meal into a memorable dining experience that delights the senses and brings people together around a common table to share stories, laughter, and the simple joy of breaking bread with loved ones.",
    "Beneath the surface of the seemingly tranquil ocean lies a vibrant and diverse ecosystem teeming with life, from the microscopic phytoplankton that form the base of the marine food web to the massive blue whales that gracefully glide through the depths, all interconnected in a delicate balance that is increasingly threatened by human activities such as pollution, overfishing, and climate change.",
    "The study of ancient civilizations offers a fascinating glimpse into the lives, beliefs, and achievements of our ancestors, revealing how they adapted to their environments, developed complex social structures, and left behind enduring monuments, artifacts, and written records that continue to inspire and intrigue archaeologists, historians, and anyone with a curiosity about the rich tapestry of human history.",
    "Mastering a new programming language involves more than just memorizing syntax and standard libraries; it requires a shift in mindset to embrace the underlying paradigms, design patterns, and idiomatic practices that define the language's ecosystem, enabling developers to write elegant, efficient, and maintainable code that solves real-world problems while collaborating effectively with other members of the software engineering community.",
    "The protagonist of the epic fantasy novel embarked on a perilous quest across a continent fraught with danger, encountering mythical creatures, uncovering ancient prophecies, and forming unlikely alliances with a diverse cast of characters, all while grappling with their own inner demons and discovering the true meaning of courage, sacrifice, and destiny in the face of overwhelming odds and an encroaching darkness."
];

/// Returns a random sentence of the requested length.
pub fn get_sentence(length: SentenceLength) -> &'static str {
    let mut rng = rand::rng();
    match length {
        SentenceLength::Short => SHORT_SENTENCES.choose(&mut rng).unwrap(),
        SentenceLength::Medium => MEDIUM_SENTENCES.choose(&mut rng).unwrap(),
        SentenceLength::Long => LONG_SENTENCES.choose(&mut rng).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_sentence_returns_non_empty() {
        assert!(!get_sentence(SentenceLength::Short).is_empty());
        assert!(!get_sentence(SentenceLength::Medium).is_empty());
        assert!(!get_sentence(SentenceLength::Long).is_empty());
    }

    #[test]
    fn sentence_length_from_str() {
        assert_eq!("short".parse::<SentenceLength>(), Ok(SentenceLength::Short));
        assert_eq!("Medium".parse::<SentenceLength>(), Ok(SentenceLength::Medium));
        assert_eq!("LONG".parse::<SentenceLength>(), Ok(SentenceLength::Long));
        assert!("invalid".parse::<SentenceLength>().is_err());
    }
}
