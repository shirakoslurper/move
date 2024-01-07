### notes:

> To preface, this was not a successful project owing to my lack of domain knowledge concerning what I was attempting to achieve.

While some ways of making money on-chain are known and oft exploited, like arbitrage or liquidations, there are other ways that have yet to be discovered. These can range from exploiting contract vulnerabilities to stranger opportunities on contracts people would never consider.

Mmm. Any security researcher would probably be cocking their head at this. Hmm, this sounds familiar. Hey wait a minute can't we just... YES.

Yes, you can use vulnerability scanning and verification tools to find these opportunities. Set the pre-conditions to what the chain reflects at the moment. Set the post-conditions to the opposite of what you want, say "new balance <= old balance" and let the tools find counterexamples.

Here's the earliest example I could find of this being done: [How to steal Ethers: scanning for vulnerable contracts](https://www.palkeo.com/en/projets/ethereum/stealing_ether.html).

> This utilizes a unique vulnerability scanning tool (Pakala) built by the author that it allows for the stacking of calls. So, if a sequence of calls is required to advance the state to a place where we can get money out, it can generate it.

Issue is, that this is easier said than done. The above example scans only for the most basic of vulnerabilities. And it takes a hot minute of compute.

When I ran across this stuff, I remembered my days of exploring the possibilities of MEV on Aptos. It was a brief snippet about Move, the smart contract language of the chain, being easy to verify.

> Due to limitations of the SDK (how we could interact with the chain) and other things, I decided against pursuing MEV on Aptos originally.

Wow. Perfect.

> Not really. Besides what I would eventually realize, the chain was too new to even have many long-tail, weird opportunities worth exploiting.

I thought that it would be as simple as downloading the contract bytecode and using the Move verifier. Except it wasn't. The programs are usually written with a spec (for verification) then compiled. I've got all the particulars in my notes, but in short, if we wanted to add a spec AFTER compilation we would have to backtrack a couple compilatilation steps. Thankfully there was a dissassembler available but getting everything back into AST was something I had to take care of.

And then when playing around with Dafny, a verification aware programming language, that uses Boogie in the backend same as with the Move verifier I realized an issue. I wanted to see if I could use the same principles as for Symbolic MEV to generate sequences of gene mutations to get from one gene to another. But despite everything I tried I was failing all of my test cases. I was getting spurious counterexamples. 

After a bit of research I learned that this was probably due to the incompleteness of SMT solvers and with the "axiomatization of several data structures inclusing sequences" to be incomplete. Ah. How sad.

The genome problem was a very simple test case the tool I was meaning to use could not deal with. So after finishing the bytecode to AST tool I dropped everything.

> There were also a bunch of other glaring problems. How specific and correct the specs had to be. How the verifier assumed the correctness of the specs of callee functions. The difficulty of writing proper loop invariants. How functions are verified individually. The difficulty of "stacking outcomes" (I thought the equivalent for a deductive verifier would be working backwards, but it's not that simple). And here I assumed that a lot of this already was or could be automated. But that itself is a hard problem I do not have the skillset to solve. But even if I had, I would've run into the limitations of the underlying verification tool.

But I did get a peek into the Move compiler and that was cool. And I got humbled (happens often), which was cooler.

The bulk of the work I did is in `language/move-bytecode-prover`.

All my notes (that I could find) are in `bytecode-prover-notes`.

[![License](https://img.shields.io/badge/license-Apache-green.svg)](LICENSE)
[![Discord chat](https://img.shields.io/discord/903339070925721652.svg?logo=discord&style=flat-square)](https://discord.gg/M95qX3KnG8)

![Move logo](assets/color/SVG/Move_Logo_Design_Digital_Final_-01.svg)

# The Move Language

Move is a programming language for writing safe smart contracts originally developed at Facebook to power the Diem blockchain. Move is designed to be a platform-agnostic language to enable common libraries, tooling, and developer communities across diverse blockchains with vastly different data and execution models. Move's ambition is to become the "JavaScript of web3" in terms of ubiquity--when developers want to quickly write safe code involving assets, it should be written in Move.

This repository is the official home of the Move virtual machine, bytecode verifier, compiler, prover, package manager, and book. For Move code examples and papers, check out [awesome-move](https://github.com/MystenLabs/awesome-move).

## Quickstart

### Build the [Docker](https://www.docker.com/community/open-source/) Image for the Command Line Tool

```
docker build -t move/cli -f docker/move-cli/Dockerfile .
```

### Build a Test Project

```
cd ./language/documentation/tutorial/step_1/BasicCoin
docker run -v `pwd`:/project move/cli build
```

Follow the [language/documentation/tutorial](./language/documentation/tutorial/README.md) to set up move for development.

## Community

* Join us on the [Move Discord](https://discord.gg/cPUmhe24Mz).
* Browse code and content from the community at [awesome-move](https://github.com/MystenLabs/awesome-move).

## License

Move is licensed as [Apache 2.0](https://github.com/move-language/move/blob/main/LICENSE).
