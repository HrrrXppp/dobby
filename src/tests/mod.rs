#[test]
fn test_create() {
    let mut app = App{};
    let mut listener = Listener::new();
    let listener_pid = app.create(listener);
    assert_ne!( listener_pid, Pid::from_raw( 0 ) );
/*
    let mq = PosixMq::create("/hello_posixmq").unwrap();
    mq.send(0, b"message").unwrap();
    // messages with equal priority will be received in order
    mq.send(0, b"queue").unwrap();
    // but this message has higher priority and will be received first
    mq.send(10, b"Hello,").unwrap();
*/

}
