Some time software need work with clock. This part can be hard/long/unstable for test with real clock.
For example test about thread will sleep a hour take... a hour.

Fake time solve this problem: It proxy calls to real time subsystem on production and allow to manage time in tests.

