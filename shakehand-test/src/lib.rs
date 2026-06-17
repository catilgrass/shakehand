#[cfg(test)]
mod test {
    use crate::locale::{Global, Languages, set_lang};
    use std::sync::{Mutex, MutexGuard};

    /// Global lock to serialize tests that mutate `__SHAKE_HAND_LANG`.
    /// Without this, parallel test execution causes races on the shared global atomic state.
    static LANG_LOCK: Mutex<()> = Mutex::new(());
    fn lock_lang() -> MutexGuard<'static, ()> {
        LANG_LOCK.lock().unwrap()
    }

    #[test]
    fn test_english() {
        let _lock = lock_lang();
        set_lang(Languages::en);
        assert_eq!(Global::world(), "world");
        assert_eq!(Global::greeting("Alice"), "Hello, Alice!");
        assert_eq!(Global::farewell("Bob"), "Goodbye, Bob!");
        assert_eq!(Global::thanks(), "Thank you!");
    }

    #[test]
    fn test_chinese() {
        let _lock = lock_lang();
        set_lang(Languages::zh_CN);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("小明"), "你好，小明！");
        assert_eq!(Global::farewell("小红"), "再见，小红！");
        assert_eq!(Global::thanks(), "谢谢！");
    }

    #[test]
    fn test_japanese() {
        let _lock = lock_lang();
        set_lang(Languages::ja);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("田中"), "こんにちは、田中！");
        assert_eq!(Global::farewell("佐藤"), "さようなら、佐藤！");
        assert_eq!(Global::thanks(), "ありがとうございます！");
    }

    #[test]
    fn test_korean() {
        let _lock = lock_lang();
        set_lang(Languages::ko);
        assert_eq!(Global::world(), "세계");
        assert_eq!(Global::greeting("철수"), "안녕하세요, 철수님！");
        assert_eq!(Global::farewell("영희"), "안녕히 가세요, 영희님！");
        assert_eq!(Global::thanks(), "감사합니다！");
    }

    #[test]
    fn test_french() {
        let _lock = lock_lang();
        set_lang(Languages::fr);
        assert_eq!(Global::world(), "monde");
        assert_eq!(Global::greeting("Marie"), "Bonjour, Marie !");
        assert_eq!(Global::farewell("Pierre"), "Au revoir, Pierre !");
        assert_eq!(Global::thanks(), "Merci !");
    }

    #[test]
    fn test_german() {
        let _lock = lock_lang();
        set_lang(Languages::de);
        assert_eq!(Global::world(), "Welt");
        assert_eq!(Global::greeting("Hans"), "Hallo, Hans!");
        assert_eq!(Global::farewell("Greta"), "Auf Wiedersehen, Greta!");
        assert_eq!(Global::thanks(), "Danke!");
    }

    #[test]
    fn test_spanish() {
        let _lock = lock_lang();
        set_lang(Languages::es);
        assert_eq!(Global::world(), "mundo");
        assert_eq!(Global::greeting("Carlos"), "¡Hola, Carlos!");
        assert_eq!(Global::farewell("Lucía"), "¡Adiós, Lucía!");
        assert_eq!(Global::thanks(), "¡Gracias!");
    }

    #[test]
    fn test_russian() {
        let _lock = lock_lang();
        set_lang(Languages::ru);
        assert_eq!(Global::world(), "мир");
        assert_eq!(Global::greeting("Анна"), "Привет, Анна!");
        assert_eq!(Global::farewell("Иван"), "До свидания, Иван!");
        assert_eq!(Global::thanks(), "Спасибо!");
    }

    #[test]
    fn test_arabic() {
        let _lock = lock_lang();
        set_lang(Languages::ar);
        assert_eq!(Global::world(), "عالم");
        assert_eq!(Global::greeting("أحمد"), "مرحبًا، أحمد!");
        assert_eq!(Global::farewell("فاطمة"), "وداعًا، فاطمة!");
        assert_eq!(Global::thanks(), "شكرًا!");
    }

    #[test]
    fn test_portuguese() {
        let _lock = lock_lang();
        set_lang(Languages::pt);
        assert_eq!(Global::world(), "mundo");
        assert_eq!(Global::greeting("João"), "Olá, João!");
        assert_eq!(Global::farewell("Maria"), "Tchau, Maria!");
        assert_eq!(Global::thanks(), "Obrigado!");
    }

    #[test]
    fn test_fallback_zh_hk_to_zh_cn() {
        let _lock = lock_lang();
        set_lang(Languages::zh_HK);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("小明"), "你好，小明！");
        assert_eq!(Global::farewell("小红"), "再见，小红！");
        assert_eq!(Global::thanks(), "谢谢！");
    }

    #[test]
    fn test_fallback_zh_tw_to_zh_cn() {
        let _lock = lock_lang();
        set_lang(Languages::zh_TW);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("小明"), "你好，小明！");
    }

    #[test]
    fn test_fallback_vi_to_zh_cn() {
        let _lock = lock_lang();
        set_lang(Languages::vi);
        assert_eq!(Global::world(), "世界");
        assert_eq!(Global::greeting("小明"), "你好，小明！");
    }

    #[test]
    fn test_fallback_ms_to_id_to_en() {
        let _lock = lock_lang();
        set_lang(Languages::ms);
        assert_eq!(Global::world(), "world");
        assert_eq!(Global::greeting("Alice"), "Hello, Alice!");
    }

    #[test]
    fn test_fallback_it_to_fr() {
        let _lock = lock_lang();
        set_lang(Languages::it);
        assert_eq!(Global::world(), "monde");
        assert_eq!(Global::greeting("Marie"), "Bonjour, Marie !");
    }

    #[test]
    fn test_fallback_pt_to_es() {
        let _lock = lock_lang();
        // pt has direct translations, so fallback should not be triggered
        set_lang(Languages::pt);
        assert_eq!(Global::world(), "mundo");
        assert_eq!(Global::greeting("João"), "Olá, João!");
    }

    #[test]
    fn test_fallback_tr_to_ar() {
        let _lock = lock_lang();
        set_lang(Languages::tr);
        assert_eq!(Global::world(), "عالم");
        assert_eq!(Global::greeting("أحمد"), "مرحبًا، أحمد!");
    }

    #[test]
    fn test_fallback_parameterless_translation_as_param() {
        let _lock = lock_lang();
        set_lang(Languages::en);
        let greeting = Global::greeting(Global::world());
        assert_eq!(greeting, "Hello, world!");
    }

    #[test]
    fn test_set_lang_from_str() {
        let _lock = lock_lang();
        set_lang("en");
        assert_eq!(Global::world(), "world");

        set_lang("zh_CN");
        assert_eq!(Global::world(), "世界");

        set_lang("zh-CN");
        assert_eq!(Global::world(), "世界");

        set_lang("  en  ");
        assert_eq!(Global::world(), "world");
    }

    #[test]
    fn test_set_lang_from_string() {
        let _lock = lock_lang();
        set_lang(String::from("fr"));
        assert_eq!(Global::world(), "monde");

        set_lang(String::from("zh_TW"));
        assert_eq!(Global::world(), "世界");
    }

    #[test]
    fn test_languages_display() {
        let _lock = lock_lang();

        assert_eq!(Languages::en.to_string(), "en");
        assert_eq!(Languages::zh_CN.to_string(), "zh_CN");
        assert_eq!(Languages::fr.to_string(), "fr");
        assert_eq!(format!("{}", Languages::ja), "ja");
    }

    #[test]
    fn test_from_str_cases() {
        use std::convert::From;

        assert_eq!(
            <Languages as From<&str>>::from("en") as u8,
            Languages::en as u8
        );
        assert_eq!(
            <Languages as From<&str>>::from("zh_CN") as u8,
            Languages::zh_CN as u8
        );
        assert_eq!(
            <Languages as From<&str>>::from("zh-CN") as u8,
            Languages::zh_CN as u8
        );
        assert_eq!(
            <Languages as From<&str>>::from("ZH_CN") as u8,
            Languages::zh_CN as u8
        );
        assert_eq!(
            <Languages as From<&str>>::from("zh-cn") as u8,
            Languages::zh_CN as u8
        );
    }

    #[test]
    #[should_panic(expected = "unknown language")]
    fn test_from_str_unknown() {
        let _ = Languages::from("klingon");
    }
}

mod locale {
    shakehand::locale!("./locale");
}
