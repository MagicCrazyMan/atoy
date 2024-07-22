# Diary

## 2024/7/21

### App

- App is the first structure to be created when the application is started.
- App has a message channel, an App instance will share this message around the application.

Forget everything else, let's build up the message channel first.
I do think `crossbeam-channel` is a good way to achieve the message channel, but i am not sure that whether it is a good idea to make an async program over WebAssembly.

<!-- ### Resource Keys

- Resource keys are used to identify resources in the application.
- Resource keys is an `enum` with types of `usize`, `` -->
