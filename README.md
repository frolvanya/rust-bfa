To compile program:
```
$ cargo build --release
```
To start program (generating own password list): 
```
$ ./target/release/rust-bfa --url "https://website.com" --login "username" --username-field "username" --password-field "password" --err "Auth Error"
```
To start program (reading file with password list): 
```
$ ./target/release/rust-bfa --url "https://website.com" --login "username" --username-field "username" --password-field "password" --err "Auth Error" --file "file_path"
```