pub trait Parser{

    fn real_message_by_get<'lifetime>( &self, message: &'lifetime str ) -> ( &'lifetime str, &'lifetime str ) {
        println!(  "real_message_by_get {}", message );
        let offset = message.find("HTTP").unwrap_or( message.len() );
        let (mut first, _last) = message.split_at(offset);
        println!(  "    first {}", first );
        if first.len() <= 1 {
            return ( "", "" );
        }
        first = first.trim();
        first = &first[1..];
        first = first.trim();
        let offset1 = first.find( "/" ).unwrap_or( first.len() );
        let ( func, mut args ) = first.split_at(offset1);
        if args.len() > 0 {
            args = &args[1..];
        }
        return ( func, args );
    }

}
