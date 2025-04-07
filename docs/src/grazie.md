# Why grazie?  
Development on `grazie` began when a need for a more extensible HTTP server solution was required.

_(In particular, I was working on an in-house Google OAuth handler for a backend API, and I began to have problems with 
the way middleware was handled in my choice HTTP framework. Each step of the way implementing authorization to different
pages felt like one roadblock was being hit after another. `grazie`'s goal is to serve a more Rust-idiomatic HTTP 
framework capable of handling a heavier workload, with greater control over how a user's request is responded to.)_