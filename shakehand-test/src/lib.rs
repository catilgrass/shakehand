#[cfg(test)]
mod test {
    use crate::locale::{Global, Languages, set_lang};

    #[test]
    fn test_english() {
        set_lang(Languages::en);
        assert_eq!(Global::world(), "world");
        assert_eq!(Global::greeting("Alice"), "Hello, Alice!");
        assert_eq!(Global::farewell("Bob"), "Goodbye, Bob!");
        assert_eq!(Global::thanks(), "Thank you!");
    }

    #[test]
    fn test_chinese() {
        set_lang(Languages::zh_CN);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("小明"), "你好，小明！");
        assert_eq!(Global::farewell("小红"), "再见，小红！");
        assert_eq!(Global::thanks(), "谢谢！");
    }

    #[test]
    fn test_japanese() {
        set_lang(Languages::ja);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("田中"), "こんにちは、田中！");
        assert_eq!(Global::farewell("佐藤"), "さようなら、佐藤！");
        assert_eq!(Global::thanks(), "ありがとうございます！");
    }

    #[test]
    fn test_korean() {
        set_lang(Languages::ko);
        assert_eq!(Global::world(), "세계");
        assert_eq!(Global::greeting("철수"), "안녕하세요, 철수님！");
        assert_eq!(Global::farewell("영희"), "안녕히 가세요, 영희님！");
        assert_eq!(Global::thanks(), "감사합니다！");
    }

    #[test]
    fn test_french() {
        set_lang(Languages::fr);
        assert_eq!(Global::world(), "monde");
        assert_eq!(Global::greeting("Marie"), "Bonjour, Marie !");
        assert_eq!(Global::farewell("Pierre"), "Au revoir, Pierre !");
        assert_eq!(Global::thanks(), "Merci !");
    }

    #[test]
    fn test_german() {
        set_lang(Languages::de);
        assert_eq!(Global::world(), "Welt");
        assert_eq!(Global::greeting("Hans"), "Hallo, Hans!");
        assert_eq!(Global::farewell("Greta"), "Auf Wiedersehen, Greta!");
        assert_eq!(Global::thanks(), "Danke!");
    }

    #[test]
    fn test_spanish() {
        set_lang(Languages::es);
        assert_eq!(Global::world(), "mundo");
        assert_eq!(Global::greeting("Carlos"), "¡Hola, Carlos!");
        assert_eq!(Global::farewell("Lucía"), "¡Adiós, Lucía!");
        assert_eq!(Global::thanks(), "¡Gracias!");
    }

    #[test]
    fn test_russian() {
        set_lang(Languages::ru);
        assert_eq!(Global::world(), "мир");
        assert_eq!(Global::greeting("Анна"), "Привет, Анна!");
        assert_eq!(Global::farewell("Иван"), "До свидания, Иван!");
        assert_eq!(Global::thanks(), "Спасибо!");
    }

    #[test]
    fn test_arabic() {
        set_lang(Languages::ar);
        assert_eq!(Global::world(), "عالم");
        assert_eq!(Global::greeting("أحمد"), "مرحبًا، أحمد!");
        assert_eq!(Global::farewell("فاطمة"), "وداعًا، فاطمة!");
        assert_eq!(Global::thanks(), "شكرًا!");
    }

    #[test]
    fn test_portuguese() {
        set_lang(Languages::pt);
        assert_eq!(Global::world(), "mundo");
        assert_eq!(Global::greeting("João"), "Olá, João!");
        assert_eq!(Global::farewell("Maria"), "Tchau, Maria!");
        assert_eq!(Global::thanks(), "Obrigado!");
    }

    #[test]
    fn test_parameterless_translation_as_param() {
        set_lang(Languages::en);
        let greeting = Global::greeting(Global::world());
        assert_eq!(greeting, "Hello, world!");
    }
}

mod locale {
    shakehand::locale!("./locale");
}
