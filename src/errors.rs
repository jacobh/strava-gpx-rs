error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Xml(::xmltree::ParseError);
    }
}
