use anyhow::Context;
use async_trait::async_trait;
use futures::future::try_join3;
use futures::TryFutureExt;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[async_trait]
pub trait Processor: Send + Sync {
    fn process(&mut self, line: String) -> anyhow::Result<String>;

    async fn flush(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Executor {
    out_processors: Vec<Box<dyn Processor + Send + Sync>>,
    err_processors: Vec<Box<dyn Processor + Send + Sync>>,
}

impl Executor {
    pub fn push_out<P: Processor + Send + Sync + 'static>(&mut self, processor: P) {
        self.out_processors
            .push(Box::new(processor) as Box<dyn Processor + Send + Sync>);
    }

    pub fn push_err<P: Processor + Send + Sync + 'static>(&mut self, processor: P) {
        self.err_processors
            .push(Box::new(processor) as Box<dyn Processor + Send + Sync>);
    }

    pub async fn exec<P>(mut self, cmd: &[String], workdir: P) -> anyhow::Result<()>
    where
        P: AsRef<Path> + std::fmt::Debug,
    {
        let capture_out = !self.out_processors.is_empty();
        let capture_err = !self.err_processors.is_empty();

        let mut child = Command::new(&cmd[0]); // TODO: move the program/args split beforehand
        let child = child.args(&cmd[1..]).current_dir(workdir.as_ref());

        if capture_out {
            child.stdout(Stdio::piped());
        }
        if capture_err {
            child.stderr(Stdio::piped());
        }

        let mut child = child
            .spawn()
            .with_context(|| format!("could not spawn {:?} in {:?}", &cmd, &workdir))?;

        let child_stdout = child.stdout.take();
        let process_out = tokio::spawn(async move {
            if capture_out {
                if let Some(stdout) = child_stdout {
                    let mut out_reader = BufReader::new(stdout).lines();

                    while let Some(mut line) = out_reader.next_line().await? {
                        for processor in &mut self.out_processors {
                            line = processor.process(line)?;
                        }
                        println!("{}", line);
                    }

                    for processor in &mut self.out_processors {
                        processor.flush().await?;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        let child_stderr = child.stderr.take();
        let process_err = tokio::spawn(async move {
            if capture_err {
                if let Some(stderr) = child_stderr {
                    let mut err_reader = BufReader::new(stderr).lines();

                    while let Some(mut line) = err_reader.next_line().await? {
                        for processor in &mut self.err_processors {
                            line = processor.process(line)?;
                        }
                        eprintln!("{}", line);
                    }

                    for processor in &mut self.err_processors {
                        processor.flush().await?;
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        let _ = try_join3(
            child.wait().map_err(anyhow::Error::msg),
            process_out.map_err(anyhow::Error::msg),
            process_err.map_err(anyhow::Error::msg),
        )
        .await?;

        Ok(())
    }
}
