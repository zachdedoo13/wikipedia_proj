#[macro_export]
macro_rules! timer {
    ($code: block) => {{
        let st = instant::Instant::now();
        let out = { $code };
        println!("Time elapsed -> {:?}", st.elapsed());
        out
    }};

    ($name: literal, $code: block) => {{
        let st = instant::Instant::now();
        let out = { $code };
        println!("{} -> {:?}", $name, st.elapsed());
        out
    }};
}

#[macro_export]
macro_rules! timer_var {
    ($code: block) => {{
        let st = instant::Instant::now();
        let out = { $code };
        (st.elapsed(), out)
    }};
}