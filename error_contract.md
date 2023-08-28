# Error Contract

## Terminology

* *interface*: Trait used to abstract the specific implementation details of some behavior of part of a program through its methods. 
* *implementation*: Concrete type that implement an *interface*.
* *interface object*: A [trait object] for an *interface* e.g. `Box<dyn Trait>`. The *implementation* is said to have been type erased.
* *client*: The user or caller of an *interface*'s methods, usually taking an *interface object* as a parameter in a pattern known as *[Dependency injection]*.
* *Dependency injection*: A programming pattern in which dependencies are provided as parameters to separate a class from the construction of those dependencies. See also [Dependency inversion principle]

[trait object]: https://doc.rust-lang.org/book/ch17-02-trait-objects.html
[Dependency inversion principle]: https://en.wikipedia.org/wiki/Dependency_inversion_principle
[Dependency injection]: https://en.wikipedia.org/wiki/Dependency_injection

## Errors returned from Interfaces

1. Fallible methods on an *interface* return `Result<T, anyhow::Error>>` to abstract away the implementation's specific Error type(s).

This gives the *implementation* freedom to choose whatever error type they deem to be appropriate, as long as it implements the std::error:Error trait.

This means the user of an *implementation* is unable to match on the specific type of error to decide how to react.

2. *Clients* report any Errors returned by an *interface* to the end user as the source of their error.

The assumption is that the errors returned are fatal - the problem must be reported to the end user as there is no way for the program to proceed.

When this assumption does not hold for an error caused by an *implementation*, the *implementation* is expected to attempt to handle it.

Otherwise an *interface* may provide some mechanism for prompting the user to respond to an expected error; but such mechanisms are not yet well defined.

**Example**

```
pub trait RemoteStorage {
    /// Download a game save file from remote storage to local storage.
    fn download(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()>;

    /// Upload a game save file from local storage to remote storage.
    fn upload(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()>;
}
```

There is a provided `ErrorWithSuggestion` type and `ErrorSuggestions` extension trait to support adding a suggestion to an error in a manner similar to anyhow's Context trait.

**Example**

```
// bring extension trait into scope
use scut::ErrorSuggestions;

impl Remote Storage for MyImplementation {
    fn download(&mut self, save: &Save, local_path: &Path) -> anyhow::Result<()> {

    }
}


```



### *Response mechanism*s for expected, recoverable errors

*TODO*

#### *Ideas*

* anyhow supports downcasting into a concrete error type
* the *client* must have code written to respond to an error
* commonly, the conclusion is therefore the *client* must define every possible kind of error and respond according to what kind of error it is
* however that quickly runs into problems when the *client* doesn't know what errors a new *implementation* might encounter
* what if the *client* instead of specifying every possible (anticipated) error, only specifies the errors it is written to handle (other than simply throwing upwards to be reported to the end user)
* the *client* creates a public type for such errors
* *implementation*s may depend on that type (as well as the interface) and wrap their errors in that error type - still returning an anyhow::Result
* the *client*'s error handling code attempts to downcast the *implementation*'s error into its type of known, handle-able errors
  * if it downcasts, handle it!
  * if not, throw it!
* this is nice, but still has a problem when an *implementation* encounters an error that doesn't "fit" into the *client*'s public handle-able error type.

* So, another approach is to instead for the *client* to provide a generic `HandledError` type for *implementation*s to use
* `HandledError`` describes the general approach to error handling that the *client* should use for the error raised by the *implementation*
  * It also includes the original error should the client decide to raise the error instead of (or after failing to) handle it as suggested by the *implementation*
  * suggested variants for HandledError:
    * RetryAfter(Duration) -> retry after Duration has expired (e.g. API request has been throttled)
    * ManualRetry -> have the user try again at an unspecified future time (e.g. server busy or intermittent server failure)
    * RetryAt(DateTime) -> retry at a specified future date (e.g. server is undergoing maintenance) - note: the *client* must decide if it is willing to wait long enough, a cli might refuse to wait more than a few seconds and instead report this to the user - that's fine!
    * ResourceNotFound -> The *client* might choose to try a fallback location/name for whatever it is they're looking for?
    * ... I notice that I'm reusing a lot of HTTP Status Code type terminology, not too surprising really!
    * possibly the Retry variants are the only ones needed?

---

## Error reporting

### Example error messages

* "unexpected end of file"
* 
* "invalid IP address syntax"
* "second time provided was later than self"
* "invalid UTF-8 sequence of {} bytes from index {}"
* "environment variable was not valid unicode: {:?}"

In addition, we define *Suggestion*: user facing advice to avoid future errors.
*Suggestion*s should be formatted as a full sentence including capitalisation and full stops.
This is OK because they will always be displayed to the user on their own line after the full error.
*Suggestion*s should not contain newlines.

### Example Suggestions

* "Install Dropbox from https://www.dropbox.com/desktop in order to use scut."
* "Double check your config's seven_zip_path setting; does it point to a folder containing 7z.exe?"
* "Install 7z from https://www.7-zip.org/ in order to use scut."

### Wording conventions ###

The conventions depend on whether the error message is context for a following error, or the root cause error.

#### Contexts ####

* use the form **"failed to ..."** for context messages
* where "..." contains what was attempted
  * in the **present tense**
  * in a form that follows "if not for this error, it would ..." grammatically

**Examples**

* "failed to download data"
* "failed to detach the important thing"
* "failed to read instrs from /path/to/instrs.json"

#### Errors ####

* try to describe what caused the error, or the reason for the operation (described by the context) failed
  * word your error message such that it fits into "*<context>* because *<error>*" structure
  * e.g. "connection reset by peer" fits into "*failed to download the thing* because *connection reset by peer*
* be concise
  * but include important details
  * **do not** add introductory words or phrases such as "due to... " or "because ..." "since..." or anything like that
* ubiquitous third party errors should be used as-is, since their familiarity is valuable for end users
    * do not feel compelled to replace all errors to conform with this error formatting guidance
* resist the temptation to capitalise the error message

**Examples**

* "provided string was not `true` or `false`"
* "dropbox not configured"
* "unexpected end of file"
* "No such file or directory (os error 2)" - from [`io::Error`], regrettably capitalised but too late to change now!
* "invalid header (expected {expected:?}, got {found:?})"

## Examples

current

```
No config file found.
Writing default config file to C:\Users\David\AppData\Roaming\scut\config.toml
Error: SCUT (Strategic Command Utility Tool) has encountered an error
Could not create default configuration file
Unable to find your dropbox folder
Attempted to create a default config for you but there was a problem
```

idealised

```
No config file found.
Writing default config file to C:\Users\David\AppData\Roaming\scut\config.toml

Error: could not create default config file

Caused by:
    * failed to find your Dropbox folder
    * Dropbox not configured

> Install Dropbox from https://www.dropbox.com/desktop in order to use scut.
```

current

> It'll allow me to download a save, and says it happens successfully, but it doesn't actually extract anything to my Hotseat folder

idealised

```
Downloading Axis 74 TG to C:\...

Error: failed to decompress C:\... to ...

Caused by:
    * failed to find 7z executable
    * file C:\...peazip\bin\7z.exe\7z.exe does not exist

> Double check your config's seven_zip_path setting, does it point to a folder containing 7z.exe?
> Install 7z from https://www.7-zip.org/ in order to use scut.
```

