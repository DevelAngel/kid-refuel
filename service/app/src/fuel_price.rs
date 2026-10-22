use leptos::prelude::*;

fn format_price(price: [u8; 3]) -> String {
    format!("{},{:02}", price[0], price[1])
}

#[component]
pub(crate) fn FuelPrice(price: [u8; 3]) -> impl IntoView {
    view! { <span>{format_price(price)}<sup class="text-[0.6em]">{price[2]}</sup></span> }
}

#[cfg(test)]
mod tests {
    use super::format_price;

    #[test]
    fn pads_the_fraction_to_two_digits() {
        assert_eq!(format_price([2, 3, 9]), "2,03");
    }

    #[test]
    fn preserves_two_digit_fractions() {
        assert_eq!(format_price([1, 45, 9]), "1,45");
    }
}
