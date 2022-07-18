# Fixtures

This options was discourage as parsing datetimes and object ids was painful and implied
knowing the schema of each fixture, going against the main goal of using fixtures as json.

Instead however, the `doc!` macro is used to create objects on the fly to be used as mocked data