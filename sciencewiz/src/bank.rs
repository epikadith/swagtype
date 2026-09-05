//! Curated science question bank.

use quiz_engine::Question;
use rand::seq::SliceRandom;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScienceCategory {
    Physics,
    Chemistry,
    Biology,
    EarthScience,
    General,
}

impl FromStr for ScienceCategory {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physics" => Ok(ScienceCategory::Physics),
            "chemistry" => Ok(ScienceCategory::Chemistry),
            "biology" => Ok(ScienceCategory::Biology),
            "earth" | "earthscience" => Ok(ScienceCategory::EarthScience),
            "general" => Ok(ScienceCategory::General),
            _ => Err("invalid category"),
        }
    }
}

pub fn get_questions(category: Option<ScienceCategory>, count: usize) -> Vec<Question> {
    let mut all_questions = get_all_questions();
    
    if let Some(cat) = category {
        let cat_str = format!("{:?}", cat);
        all_questions.retain(|q| q.category.as_ref() == Some(&cat_str));
    }
    
    let mut rng = rand::rng();
    all_questions.shuffle(&mut rng);
    
    all_questions.into_iter().take(count).collect()
}

fn make_q(text: &str, options: &[&str], correct: usize, cat: ScienceCategory) -> Question {
    Question {
        text: text.to_string(),
        options: options.iter().map(|s| s.to_string()).collect(),
        correct_index: correct,
        category: Some(format!("{:?}", cat)),
    }
}

fn get_all_questions() -> Vec<Question> {
    vec![
        // Physics
        make_q("What is the speed of light in a vacuum?", &["300,000 km/s", "150,000 km/s", "1,000,000 km/s", "30,000 km/s"], 0, ScienceCategory::Physics),
        make_q("What is the SI unit of force?", &["Joule", "Newton", "Watt", "Pascal"], 1, ScienceCategory::Physics),
        make_q("Who formulated the laws of motion?", &["Albert Einstein", "Isaac Newton", "Galileo Galilei", "Nikola Tesla"], 1, ScienceCategory::Physics),
        make_q("What is the escape velocity of Earth?", &["11.2 km/s", "8.5 km/s", "15.3 km/s", "9.8 m/s"], 0, ScienceCategory::Physics),
        make_q("Which particle has a positive charge?", &["Electron", "Neutron", "Proton", "Photon"], 2, ScienceCategory::Physics),
        make_q("What force keeps planets in orbit around the Sun?", &["Electromagnetism", "Strong Nuclear Force", "Weak Nuclear Force", "Gravity"], 3, ScienceCategory::Physics),
        
        // Chemistry
        make_q("What is the boiling point of water at sea level?", &["90°C", "100°C", "120°C", "80°C"], 1, ScienceCategory::Chemistry),
        make_q("What is the chemical symbol for Gold?", &["Ag", "Au", "Fe", "Pb"], 1, ScienceCategory::Chemistry),
        make_q("What is the most abundant gas in Earth's atmosphere?", &["Oxygen", "Carbon Dioxide", "Nitrogen", "Hydrogen"], 2, ScienceCategory::Chemistry),
        make_q("What is the pH of pure water?", &["5", "7", "9", "12"], 1, ScienceCategory::Chemistry),
        make_q("Which element is a liquid at room temperature?", &["Iron", "Mercury", "Sodium", "Calcium"], 1, ScienceCategory::Chemistry),
        make_q("What is the atomic number of Carbon?", &["6", "8", "12", "14"], 0, ScienceCategory::Chemistry),

        // Biology
        make_q("What part of a plant cell converts light into food?", &["Mitochondria", "Nucleus", "Chloroplast", "Ribosome"], 2, ScienceCategory::Biology),
        make_q("What is the powerhouse of the cell?", &["Nucleus", "Mitochondria", "Golgi apparatus", "Endoplasmic reticulum"], 1, ScienceCategory::Biology),
        make_q("Which molecule carries genetic information?", &["RNA", "Protein", "DNA", "Lipid"], 2, ScienceCategory::Biology),
        make_q("What is the largest organ in the human body?", &["Liver", "Brain", "Heart", "Skin"], 3, ScienceCategory::Biology),
        make_q("How many chambers does the human heart have?", &["2", "3", "4", "5"], 2, ScienceCategory::Biology),
        make_q("What pigment gives leaves their green color?", &["Melanin", "Chlorophyll", "Carotene", "Anthocyanin"], 1, ScienceCategory::Biology),

        // Earth Science
        make_q("What is the hardest known natural mineral?", &["Quartz", "Diamond", "Topaz", "Corundum"], 1, ScienceCategory::EarthScience),
        make_q("What type of rock is formed from cooled lava?", &["Igneous", "Sedimentary", "Metamorphic", "Fossilized"], 0, ScienceCategory::EarthScience),
        make_q("What is the inner core of the Earth primarily made of?", &["Liquid magma", "Solid iron and nickel", "Compressed rock", "Water"], 1, ScienceCategory::EarthScience),
        make_q("What scale is used to measure earthquake magnitude?", &["Mohs scale", "Fahrenheit scale", "Richter scale", "Beaufort scale"], 2, ScienceCategory::EarthScience),
        make_q("What layer of the atmosphere contains the ozone layer?", &["Troposphere", "Stratosphere", "Mesosphere", "Thermosphere"], 1, ScienceCategory::EarthScience),
        
        // General
        make_q("Who is known as the father of modern physics?", &["Isaac Newton", "Albert Einstein", "Niels Bohr", "Stephen Hawking"], 1, ScienceCategory::General),
        make_q("What is the nearest star to Earth?", &["Proxima Centauri", "Sirius", "The Sun", "Alpha Centauri"], 2, ScienceCategory::General),
        make_q("Which planet is known as the Red Planet?", &["Venus", "Jupiter", "Saturn", "Mars"], 3, ScienceCategory::General),
        make_q("What is the freezing point of water in Fahrenheit?", &["0°F", "32°F", "100°F", "273°F"], 1, ScienceCategory::General),
        make_q("What is the primary source of energy for the Earth's climate system?", &["Geothermal heat", "The Sun", "Wind", "Ocean currents"], 1, ScienceCategory::General),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_questions_valid() {
        let questions = get_all_questions();
        for q in questions {
            assert!(q.is_valid());
        }
    }

    #[test]
    fn test_get_questions_count() {
        let qs = get_questions(None, 5);
        assert_eq!(qs.len(), 5);
    }

    #[test]
    fn test_get_questions_filter() {
        let qs = get_questions(Some(ScienceCategory::Physics), 10);
        assert!(qs.len() <= 10);
        for q in qs {
            assert_eq!(q.category.as_deref(), Some("Physics"));
        }
    }
}
