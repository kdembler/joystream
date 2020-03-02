mod mock;

use mock::*;

use crate::*;
use system::RawOrigin;

struct TestPostEntry {
    pub post_id: u32,
    pub text: Vec<u8>,
    pub edition_number: u32,
}

struct TestThreadEntry {
    pub thread_id: u32,
    pub title: Vec<u8>,
}

fn assert_thread_content(thread_entry: TestThreadEntry, post_entries: Vec<TestPostEntry>) {
    assert!(<ThreadById<Test>>::exists(thread_entry.thread_id));

    let actual_thread = <ThreadById<Test>>::get(thread_entry.thread_id);
    let expected_thread = Thread {
        title: thread_entry.title,
        created_at: 1,
        author_id: 1,
    };
    assert_eq!(actual_thread, expected_thread);

    for post_entry in post_entries {
        let actual_post =
            <PostThreadIdByPostId<Test>>::get(thread_entry.thread_id, post_entry.post_id);
        let expected_post = Post {
            text: post_entry.text,
            created_at: 1,
            updated_at: 1,
            author_id: 1,
            thread_id: thread_entry.thread_id,
            edition_number: post_entry.edition_number,
        };

        assert_eq!(actual_post, expected_post);
    }
}

struct DiscussionFixture {
    pub title: Vec<u8>,
    pub origin: RawOrigin<u64>,
}

impl Default for DiscussionFixture {
    fn default() -> Self {
        DiscussionFixture {
            title: b"title".to_vec(),
            origin: RawOrigin::Signed(1),
        }
    }
}

impl DiscussionFixture {
    fn with_title(self, title: Vec<u8>) -> Self {
        DiscussionFixture { title, ..self }
    }
}

struct PostFixture {
    pub text: Vec<u8>,
    pub origin: RawOrigin<u64>,
    pub thread_id: u32,
    pub post_id: Option<u32>,
}

impl PostFixture {
    fn default_for_thread(thread_id: u32) -> Self {
        PostFixture {
            text: b"text".to_vec(),
            thread_id,
            origin: RawOrigin::Signed(1),
            post_id: None,
        }
    }

    fn with_text(self, text: Vec<u8>) -> Self {
        PostFixture { text, ..self }
    }

    fn with_origin(self, origin: RawOrigin<u64>) -> Self {
        PostFixture { origin, ..self }
    }

    fn change_thread_id(self, thread_id: u32) -> Self {
        PostFixture { thread_id, ..self }
    }

    fn change_post_id(self, post_id: u32) -> Self {
        PostFixture {
            post_id: Some(post_id),
            ..self
        }
    }

    fn add_post_and_assert(&mut self, result: Result<(), &'static str>) -> Option<u32> {
        let add_post_result = Discussions::add_post(
            self.origin.clone().into(),
            self.thread_id,
            self.text.clone(),
        );

        assert_eq!(add_post_result, result);

        if result.is_ok() {
            self.post_id = Some(<PostCount>::get());
        }

        self.post_id
    }

    fn update_post_with_text_and_assert(
        &mut self,
        new_text: Vec<u8>,
        result: Result<(), &'static str>,
    ) {
        let add_post_result = Discussions::update_post(
            self.origin.clone().into(),
            self.thread_id,
            self.post_id.unwrap(),
            new_text,
        );

        assert_eq!(add_post_result, result);
    }

    fn update_post_and_assert(&mut self, result: Result<(), &'static str>) {
        self.update_post_with_text_and_assert(self.text.clone(), result);
    }
}

impl DiscussionFixture {
    fn create_discussion_and_assert(&self, result: Result<u32, &'static str>) -> Option<u32> {
        let create_discussion_result =
            Discussions::create_thread(self.origin.clone().into(), self.title.clone());

        assert_eq!(create_discussion_result, result);

        create_discussion_result.ok()
    }
}

#[test]
fn create_discussion_call_succeeds() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();

        discussion_fixture.create_discussion_and_assert(Ok(1));
    });
}

#[test]
fn create_post_call_succeeds() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();

        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture = PostFixture::default_for_thread(thread_id);

        post_fixture.add_post_and_assert(Ok(()));
    });
}

#[test]
fn update_post_call_succeeds() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();

        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture = PostFixture::default_for_thread(thread_id);

        post_fixture.add_post_and_assert(Ok(()));
        post_fixture.update_post_and_assert(Ok(()));
    });
}

#[test]
fn update_post_call_failes_because_of_post_edition_limit() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();

        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture = PostFixture::default_for_thread(thread_id);

        post_fixture.add_post_and_assert(Ok(()));

        for _ in 1..6 {
            post_fixture.update_post_and_assert(Ok(()));
        }

        post_fixture.update_post_and_assert(Err(MSG_POST_EDITION_NUMBER_EXCEEDED));
    });
}

#[test]
fn update_post_call_failes_because_of_the_wrong_author() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();

        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture = PostFixture::default_for_thread(thread_id);

        post_fixture.add_post_and_assert(Ok(()));

        post_fixture = post_fixture.with_origin(RawOrigin::Signed(2));

        post_fixture.update_post_and_assert(Err(MSG_NOT_AUTHOR));
    });
}

#[test]
fn thread_content_check_succeeded() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();

        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture1 = PostFixture::default_for_thread(thread_id);
        let post_id1 = post_fixture1.add_post_and_assert(Ok(())).unwrap();

        let mut post_fixture2 = PostFixture::default_for_thread(thread_id);
        let post_id2 = post_fixture2.add_post_and_assert(Ok(())).unwrap();
        post_fixture1.update_post_with_text_and_assert(b"new_text".to_vec(), Ok(()));

        assert_thread_content(
            TestThreadEntry {
                thread_id,
                title: b"title".to_vec(),
            },
            vec![
                TestPostEntry {
                    post_id: post_id1,
                    text: b"new_text".to_vec(),
                    edition_number: 1,
                },
                TestPostEntry {
                    post_id: post_id2,
                    text: b"text".to_vec(),
                    edition_number: 0,
                },
            ],
        );
    });
}

#[test]
fn create_discussion_call_with_bad_title_failed() {
    initial_test_ext().execute_with(|| {
        let mut discussion_fixture = DiscussionFixture::default().with_title(Vec::new());
        discussion_fixture.create_discussion_and_assert(Err(crate::MSG_EMPTY_TITLE_PROVIDED));

        discussion_fixture = DiscussionFixture::default().with_title([0; 201].to_vec());
        discussion_fixture.create_discussion_and_assert(Err(crate::MSG_TOO_LONG_TITLE));
    });
}

#[test]
fn add_post_call_with_invalid_thread_failed() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();
        discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture = PostFixture::default_for_thread(2);
        post_fixture.add_post_and_assert(Err(MSG_THREAD_DOESNT_EXIST));
    });
}

#[test]
fn update_post_call_with_invalid_post_failed() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();
        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture1 = PostFixture::default_for_thread(thread_id);
        post_fixture1.add_post_and_assert(Ok(())).unwrap();

        let mut post_fixture2 = post_fixture1.change_post_id(2);
        post_fixture2.update_post_and_assert(Err(MSG_POST_DOESNT_EXIST));
    });
}

#[test]
fn update_post_call_with_invalid_thread_failed() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();
        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture1 = PostFixture::default_for_thread(thread_id);
        post_fixture1.add_post_and_assert(Ok(())).unwrap();

        let mut post_fixture2 = post_fixture1.change_thread_id(2);
        post_fixture2.update_post_and_assert(Err(MSG_THREAD_DOESNT_EXIST));
    });
}

#[test]
fn add_post_call_with_invalid_text_failed() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();
        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture1 = PostFixture::default_for_thread(thread_id).with_text(Vec::new());
        post_fixture1.add_post_and_assert(Err(MSG_EMPTY_POST_PROVIDED));

        let mut post_fixture2 =
            PostFixture::default_for_thread(thread_id).with_text([0; 2001].to_vec());
        post_fixture2.add_post_and_assert(Err(MSG_TOO_LONG_POST));
    });
}

#[test]
fn update_post_call_with_invalid_text_failed() {
    initial_test_ext().execute_with(|| {
        let discussion_fixture = DiscussionFixture::default();
        let thread_id = discussion_fixture
            .create_discussion_and_assert(Ok(1))
            .unwrap();

        let mut post_fixture1 = PostFixture::default_for_thread(thread_id);
        post_fixture1.add_post_and_assert(Ok(()));

        let mut post_fixture2 = post_fixture1.with_text(Vec::new());
        post_fixture2.update_post_and_assert(Err(MSG_EMPTY_POST_PROVIDED));

        let mut post_fixture3 = post_fixture2.with_text([0; 2001].to_vec());
        post_fixture3.update_post_and_assert(Err(MSG_TOO_LONG_POST));
    });
}