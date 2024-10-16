//! Defines the `Pipeline` struct and related components for processing blockchain data updates.
//!
//! The `Pipeline` module is central to the `carbon-core` framework, offering a flexible and extensible
//! data processing architecture that supports various blockchain data types, including account updates,
//! transaction details, and account deletions. The pipeline integrates multiple data sources and
//! processing pipes to handle and transform incoming data, while recording performance metrics for
//! monitoring and analysis.
//!
//! # Overview
//!
//! This module provides the `Pipeline` struct, which orchestrates data flow from multiple sources,
//! processes it through designated pipes, and captures metrics at each stage. The pipeline is highly
//! customizable and can be configured with various components to suit specific data handling requirements.
//!
//! ## Key Components
//!
//! - **Datasources**: Provide raw data updates, which may include account or transaction details.
//! - **Account, Instruction, and Transaction Pipes**: Modular units that decode and process specific
//!   types of data. Account pipes handle account updates, instruction pipes process instructions
//!   within transactions, and transaction pipes manage complete transaction records.
//! - **Metrics**: Collects data on pipeline performance, such as processing times and error rates,
//!   providing insights into operational efficiency.
//!
//! # Fields and Configuration
//!
//! - **datasources**: A list of `Datasource` objects that act as the sources for account and transaction data.
//! - **account_pipes**: A collection of pipes for processing account updates.
//! - **account_deletion_pipes**: Pipes responsible for handling account deletion events.
//! - **instruction_pipes**: Used to process instructions within transactions.
//! - **transaction_pipes**: For handling full transactions.
//! - **metrics**: A vector of `Metrics` implementations that gather and report on performance data.
//! - **metrics_flush_interval**: Specifies how frequently metrics are flushed. Defaults to 5 seconds if unset.
//!
//! ## Notes
//!
//! - Each pipe and data source must implement the appropriate traits (`Datasource`, `AccountPipes`, `Metrics`, etc.).
//! - The `Pipeline` is designed for concurrent operation, with `Arc` and `Box` wrappers ensuring safe, shared access.
//! - Proper metric collection and flushing are essential for monitoring pipeline performance, especially in production environments.

use crate::{
    account::{AccountDecoder, AccountMetadata, AccountPipe, AccountPipes, DecodedAccount},
    account_deletion::{AccountDeletionPipe, AccountDeletionPipes},
    collection::InstructionDecoderCollection,
    datasource::{AccountDeletion, Datasource, Update, UpdateType},
    error::{CarbonResult, Error},
    instruction::{
        DecodedInstruction, InstructionDecoder, InstructionMetadata, InstructionPipe,
        InstructionPipes, NestedInstruction,
    },
    metrics::Metrics,
    processor::Processor,
    schema::TransactionSchema,
    transaction::{TransactionPipe, TransactionPipes},
    transformers,
};
use core::time;
use serde::de::DeserializeOwned;
use std::{sync::Arc, time::Instant};

/// Represents the primary data processing pipeline in the `carbon-core` framework.
///
/// The `Pipeline` struct is responsible for orchestrating the flow of data from various
/// sources, processing it through multiple pipes (for accounts, transactions, instructions,
/// and account deletions), and recording metrics at each stage. This flexible design
/// allows for customized data processing, handling a variety of update types with minimal
/// boilerplate code.
///
/// ## Overview
///
/// A `Pipeline` instance includes collections of data sources and processing pipes, enabling
/// users to configure the pipeline to handle diverse types of blockchain-related data. Each
/// pipe is responsible for decoding, processing, and routing specific data types, while the
/// metrics system records relevant statistics.
///
/// ### Key Concepts
///
/// - **Datasources**: These provide the raw data, such as account updates, transaction details,
///   and account deletions.
/// - **Pipes**: Modular units that handle specific data types:
///   - `AccountPipes` for account updates.
///   - `AccountDeletionPipes` for account deletions.
///   - `InstructionPipes` for instruction data within transactions.
///   - `TransactionPipes` for entire transaction payloads.
/// - **Metrics**: Collect performance data, enabling real-time insights and efficient monitoring.
///
/// ## Fields
///
/// - `datasources`: A vector of data sources (`Datasource` implementations) that provide
///   the data for processing. Each data source must be wrapped in an `Arc` for safe,
///   concurrent access.
/// - `account_pipes`: A vector of `AccountPipes`, each responsible for handling account updates.
/// - `account_deletion_pipes`: A vector of `AccountDeletionPipes` to handle deletion events.
/// - `instruction_pipes`: A vector of `InstructionPipes` for processing instructions within
///   transactions. These pipes work with nested instructions and are generically defined
///   to support varied instruction types.
/// - `transaction_pipes`: A vector of `TransactionPipes` responsible for processing
///   complete transaction payloads.
/// - `metrics`: A vector of `Metrics` implementations to record and track performance data.
///   Each metrics instance is managed within an `Arc` to ensure thread safety.
/// - `metrics_flush_interval`: An optional interval, in seconds, defining how frequently
///   metrics should be flushed. If `None`, the default interval is used.
///
/// ## Example
///
/// ```rust
///
/// carbon_core::pipeline::Pipeline::builder()
/// .datasource(transaction_crawler)
/// .metrics(Arc::new(LogMetrics::new()))
/// .metrics(Arc::new(PrometheusMetrics::new()))
/// .instruction(
///    TestProgramDecoder,
///    TestProgramProcessor
/// )
/// .account(
///     TestProgramDecoder,
///     TestProgramAccountProcessor
/// )
/// .transaction(TEST_SCHEMA.clone(), TestProgramTransactionProcessor)
/// .account_deletions(TestProgramAccountDeletionProcessor)
/// .build()?
/// .run()
/// .await?;
/// ```
///
/// ## Notes
///
/// - Ensure that each data source and pipe implements the required traits, such as
///   `Datasource`, `AccountPipes`, and `Metrics`, as appropriate.
/// - The pipeline is designed for concurrent operation, utilizing `Arc` and `Box`
///   types to handle shared ownership and trait object storage.
/// - The `metrics_flush_interval` controls how frequently the pipeline's metrics
///   are flushed. If `None`, a default interval (usually 5 seconds) is used.
pub struct Pipeline {
    pub datasources: Vec<Arc<dyn Datasource + Send + Sync>>,
    pub account_pipes: Vec<Box<dyn AccountPipes>>,
    pub account_deletion_pipes: Vec<Box<dyn AccountDeletionPipes>>,
    pub instruction_pipes: Vec<Box<dyn for<'a> InstructionPipes<'a>>>,
    pub transaction_pipes: Vec<Box<dyn for<'a> TransactionPipes<'a>>>,
    pub metrics: Vec<Arc<dyn Metrics>>,
    pub metrics_flush_interval: Option<u64>,
}

impl Pipeline {
    /// Creates a new `PipelineBuilder` instance for constructing a `Pipeline`.
    ///
    /// The `builder` method returns a `PipelineBuilder` that allows you to configure
    /// and customize the pipeline components before building the final `Pipeline` object.
    /// This approach provides a flexible and type-safe way to assemble a pipeline
    /// by specifying data sources, processing pipes, and metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// carbon_core::pipeline::Pipeline::builder()
    /// .datasource(transaction_crawler)
    /// .metrics(Arc::new(LogMetrics::new()))
    /// .metrics(Arc::new(PrometheusMetrics::new()))
    /// .instruction(
    ///    TestProgramDecoder,
    ///    TestProgramProcessor
    /// )
    /// // ...
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `PipelineBuilder` instance with empty collections for data sources,
    /// pipes, and metrics. You can then configure each component using the builder pattern.
    ///
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder {
            datasources: Vec::new(),
            account_pipes: Vec::new(),
            account_deletion_pipes: Vec::new(),
            instruction_pipes: Vec::new(),
            transaction_pipes: Vec::new(),
            metrics: Vec::new(),
            metrics_flush_interval: None,
        }
    }

    /// Runs the `Pipeline`, processing updates from data sources and handling metrics.
    ///
    /// The `run` method initializes the pipeline’s metrics system and starts listening for
    /// updates from the configured data sources. It checks the types of updates provided
    /// by each data source to ensure that the required data types are available for
    /// processing. The method then enters a loop where it processes each update received
    /// from the data sources in turn, logging and updating metrics based on the success
    /// or failure of each operation.
    ///
    /// # How it Works
    ///
    /// - Initializes metrics and sets up an interval for periodic metric flushing.
    /// - Spawns tasks for each data source to continuously consume updates.
    /// - Processes updates according to their type (e.g., Account, Transaction, or AccountDeletion).
    /// - Records performance metrics such as update processing times, and tracks success and failure counts.
    ///
    /// # Errors
    ///
    /// The method returns an `Err` variant if:
    /// - Required update types (e.g., `AccountUpdate`, `AccountDeletion`, `Transaction`) are not
    ///   provided by any data source, causing a mismatch in expected data processing capabilities.
    /// - A data source encounters an error while consuming updates.
    /// - An error occurs during metrics flushing or processing of updates.
    ///
    /// # Example
    ///
    /// ```rust
    /// use carbon_core::Pipeline;
    ///
    /// let mut pipeline = Pipeline::builder()
    ///     .datasource(MyDatasource::new())
    ///     .metrics(MyMetrics::new())
    ///     .build()
    ///     .expect("Failed to build pipeline");
    ///
    /// // Running the pipeline asynchronously
    /// tokio::spawn(async move {
    ///     if let Err(e) = pipeline.run().await {
    ///         eprintln!("Pipeline run error: {:?}", e);
    ///     }
    /// });
    /// ```
    ///
    /// # Notes
    ///
    /// - This method is asynchronous and should be awaited within a Tokio runtime environment.
    /// - The pipeline monitors metrics and flushes them based on the configured `metrics_flush_interval`.
    /// - The `run` method operates in an infinite loop, handling updates until a termination condition occurs.
    ///
    pub async fn run(&mut self) -> CarbonResult<()> {
        self.initialize_metrics().await?;

        let (update_sender, mut update_receiver) = tokio::sync::mpsc::unbounded_channel::<Update>();

        let update_types: Vec<UpdateType> = self
            .datasources
            .iter()
            .map(|datasource| datasource.update_types())
            .flatten()
            .collect();

        if !self.account_pipes.is_empty() && !update_types.contains(&UpdateType::AccountUpdate) {
            return Err(Error::MissingUpdateTypeInDatasource(
                UpdateType::AccountUpdate,
            ));
        }

        if !self.account_deletion_pipes.is_empty()
            && !update_types.contains(&UpdateType::AccountDeletion)
        {
            return Err(Error::MissingUpdateTypeInDatasource(
                UpdateType::AccountDeletion,
            ));
        }

        if (!self.instruction_pipes.is_empty() || !self.transaction_pipes.is_empty())
            && !update_types.contains(&UpdateType::Transaction)
        {
            return Err(Error::MissingUpdateTypeInDatasource(
                UpdateType::Transaction,
            ));
        }

        for datasource in &self.datasources {
            let sender_clone = update_sender.clone();
            let datasource_clone = Arc::clone(datasource);

            tokio::spawn(async move {
                if let Err(e) = datasource_clone.consume(&sender_clone).await {
                    log::error!("Datasource consume error: {:?}", e);
                }
            });
        }

        let mut interval = tokio::time::interval(time::Duration::from_secs(
            self.metrics_flush_interval.unwrap_or(5),
        ));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.flush_metrics().await?;
                }
                update = update_receiver.recv() => {
                    match update {
                        Some(update) => {
                            self
                                .increment_counter("updates_received", 1)
                                .await?;

                            let start = Instant::now();
                            let process_result = self.process(update.clone()).await;
                            let time_taken = start.elapsed().as_millis();


                            self
                                .record_histogram("updates_processing_times", time_taken as f64)
                                .await?;

                            match process_result {
                                Ok(_) => {
                                    self
                                        .increment_counter("updates_successful", 1)
                                        .await?;

                                    log::trace!("processed update")
                                }
                                Err(error) => {
                                    self.increment_counter("updates_failed", 1).await?;

                                    log::error!("error processing update: {:?}", error)
                                }
                            };

                            self
                                .increment_counter("updates_processed", 1)
                                .await?;

                            self
                                .update_gauge("updates_queued", update_receiver.len() as f64)
                                .await?;
                        }

                        None => {
                            self.increment_counter("updates_failed", 1).await?;
                            log::error!("error processing update");

                            break
                        }
                    }

                }
            }
        }

        Ok(())
    }

    /// Processes a single update and routes it through the appropriate pipeline stages.
    ///
    /// The `process` method takes an `Update` and determines its type, then routes it
    /// through the corresponding pipes for handling account updates, transactions, or
    /// account deletions. It also records metrics for processed updates, providing
    /// insights into the processing workload and performance.
    ///
    /// ## Functionality
    ///
    /// - **Account Updates**: Passes account updates through the `account_pipes`. Each
    ///   pipe processes the account metadata and the updated account state.
    /// - **Transaction Updates**: Extracts transaction metadata and instructions, nests
    ///   them if needed, and routes them through `instruction_pipes` and `transaction_pipes`.
    /// - **Account Deletions**: Sends account deletion events through the `account_deletion_pipes`.
    ///
    /// The method also updates metrics counters for each type of update, tracking how many
    /// updates have been processed in each category.
    ///
    /// # Parameters
    ///
    /// - `update`: An `Update` variant representing the type of data received. This can be
    ///   an `Account`, `Transaction`, or `AccountDeletion`, each triggering different
    ///   processing logic within the pipeline.
    ///
    /// # Returns
    ///
    /// Returns a `CarbonResult<()>`, indicating `Ok(())` on successful processing or an
    /// error if processing fails at any stage.
    ///
    /// # Notes
    ///
    /// - This method is asynchronous and should be awaited within a Tokio runtime.
    /// - Each type of update (account, transaction, account deletion) requires its own
    ///   set of pipes, so ensure that appropriate pipes are configured based on the
    ///   data types expected from the data sources.
    /// - Metrics are recorded after each successful processing stage to track processing
    ///   volumes and identify potential bottlenecks in real-time.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the pipes fail during processing, or if an issue arises
    /// while incrementing counters or updating metrics. Handle errors gracefully to ensure
    /// continuous pipeline operation.
    ///
    async fn process(&mut self, update: Update) -> CarbonResult<()> {
        match update {
            Update::Account(account_update) => {
                let account_metadata = AccountMetadata {
                    slot: account_update.slot,
                    pubkey: account_update.pubkey,
                };

                for pipe in self.account_pipes.iter_mut() {
                    pipe.run(
                        (account_metadata.clone(), account_update.account.clone()),
                        self.metrics.clone(),
                    )
                    .await?;
                }

                self.increment_counter("account_updates_processed", 1)
                    .await?;
            }
            Update::Transaction(transaction_update) => {
                let transaction_metadata =
                    transformers::extract_transaction_metadata(&transaction_update)?;

                let instructions_with_metadata = transformers::extract_instructions_with_metadata(
                    &transaction_metadata,
                    &transaction_update,
                )?;

                let nested_instructions =
                    transformers::nest_instructions(instructions_with_metadata);

                for pipe in self.instruction_pipes.iter_mut() {
                    for nested_instruction in nested_instructions.iter().cloned() {
                        pipe.run(&nested_instruction, self.metrics.clone()).await?;
                    }
                }

                for pipe in self.transaction_pipes.iter_mut() {
                    pipe.run(&nested_instructions, self.metrics.clone()).await?;
                }

                self.increment_counter("transaction_updates_processed", 1)
                    .await?;
            }
            Update::AccountDeletion(account_deletion) => {
                for pipe in self.account_deletion_pipes.iter_mut() {
                    pipe.run(account_deletion.clone(), self.metrics.clone())
                        .await?;
                }

                self.increment_counter("account_deletions_processed", 1)
                    .await?;
            }
        };

        Ok(())
    }

    pub async fn initialize_metrics(&self) -> CarbonResult<()> {
        for metrics in self.metrics.iter() {
            metrics.initialize().await?;
        }
        Ok(())
    }
    pub async fn flush_metrics(&self) -> CarbonResult<()> {
        for metrics in self.metrics.iter() {
            metrics.flush().await?;
        }
        Ok(())
    }
    pub async fn shutdown_metrics(&self) -> CarbonResult<()> {
        for metrics in self.metrics.iter() {
            metrics.shutdown().await?;
        }
        Ok(())
    }

    pub async fn update_gauge(&self, name: &str, value: f64) -> CarbonResult<()> {
        for metrics in self.metrics.iter() {
            metrics.update_gauge(name, value).await?;
        }
        Ok(())
    }
    pub async fn increment_counter(&self, name: &str, value: u64) -> CarbonResult<()> {
        for metrics in self.metrics.iter() {
            metrics.increment_counter(name, value).await?;
        }
        Ok(())
    }
    pub async fn record_histogram(&self, name: &str, value: f64) -> CarbonResult<()> {
        for metrics in self.metrics.iter() {
            metrics.record_histogram(name, value).await?;
        }
        Ok(())
    }
}

/// A builder for constructing a `Pipeline` instance with customized data sources, processing pipes, and metrics.
///
/// The `PipelineBuilder` struct offers a flexible way to assemble a `Pipeline` by allowing
/// configuration of its components, such as data sources, account and transaction pipes,
/// deletion handling, and metrics. Using the builder pattern, you can add the desired elements
/// incrementally and then finalize with a call to `build`.
///
/// ## Overview
///
/// The `PipelineBuilder` supports the following components:
/// - **Datasources**: Sources of data updates, such as account information and transactions.
/// - **Account Pipes**: For processing account updates from data sources.
/// - **Account Deletion Pipes**: For handling account deletion updates.
/// - **Instruction Pipes**: For handling instructions associated with transactions.
/// - **Transaction Pipes**: For handling full transaction data.
/// - **Metrics**: Collects and reports performance data, such as update processing times.
/// - **Metrics Flush Interval**: Optional interval defining how often to flush metrics data.
///
/// Each component can be added through method chaining, enhancing code readability and maintainability.
///
/// # Example
///
/// ```rust
/// carbon_core::pipeline::Pipeline::builder()
/// .datasource(transaction_crawler)
/// .metrics(Arc::new(LogMetrics::new()))
/// .metrics(Arc::new(PrometheusMetrics::new()))
/// .instruction(
///    TestProgramDecoder,
///    TestProgramProcessor
/// )
/// // ...
/// ```
///
/// # Fields
///
/// - `datasources`: A collection of `Datasource` objects wrapped in `Arc` for shared ownership
///   across threads. Each `Datasource` provides updates to the pipeline.
/// - `account_pipes`: A collection of `AccountPipes` to handle account updates.
/// - `account_deletion_pipes`: A collection of `AccountDeletionPipes` for processing account deletions.
/// - `instruction_pipes`: A collection of `InstructionPipes` to process instructions in transactions.
/// - `transaction_pipes`: A collection of `TransactionPipes` to process full transaction data.
/// - `metrics`: A vector of `Metrics` implementations for tracking pipeline performance.
/// - `metrics_flush_interval`: An optional interval (in seconds) for flushing metrics data.
///   If not set, a default flush interval will be used.
///
/// # Returns
///
/// After configuring the builder, call `build` to create a `Pipeline` instance.
/// The builder will return a `CarbonResult<Pipeline>`, which will either contain the
/// configured pipeline or an error if configuration failed.
///
/// # Notes
///
/// - The builder pattern allows for method chaining, making it easy to incrementally
///   add components to the `Pipeline`.
/// - Ensure that each component matches the data and update types expected by your application.
///
pub struct PipelineBuilder {
    pub datasources: Vec<Arc<dyn Datasource + Send + Sync>>,
    pub account_pipes: Vec<Box<dyn AccountPipes>>,
    pub account_deletion_pipes: Vec<Box<dyn AccountDeletionPipes>>,
    pub instruction_pipes: Vec<Box<dyn for<'a> InstructionPipes<'a>>>,
    pub transaction_pipes: Vec<Box<dyn for<'a> TransactionPipes<'a>>>,
    pub metrics: Vec<Arc<dyn Metrics>>,
    pub metrics_flush_interval: Option<u64>,
}

impl PipelineBuilder {
    /// Creates a new `PipelineBuilder` with empty collections for datasources, pipes, and metrics.
    ///
    /// This method initializes a `PipelineBuilder` instance, allowing you to configure each component
    /// of a `Pipeline` before building it. The builder pattern offers flexibility in adding data
    /// sources, account and transaction handling pipes, deletion processing, and metrics collection
    /// features.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new();
    /// ```
    ///
    pub fn new() -> Self {
        Self {
            datasources: Vec::new(),
            account_pipes: Vec::new(),
            account_deletion_pipes: Vec::new(),
            instruction_pipes: Vec::new(),
            transaction_pipes: Vec::new(),
            metrics: Vec::new(),
            metrics_flush_interval: None,
        }
    }

    /// Adds a datasource to the pipeline.
    ///
    /// The datasource is responsible for providing updates, such as account and transaction
    /// data, to the pipeline. Multiple datasources can be added to handle various types of updates.
    ///
    /// # Parameters
    ///
    /// - `datasource`: The data source to add, implementing the `Datasource` trait.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .datasource(MyDatasource::new());
    /// ```
    ///
    pub fn datasource(mut self, datasource: impl Datasource + Send + Sync + 'static) -> Self {
        self.datasources.push(Arc::new(datasource));
        self
    }

    /// Adds an account pipe to process account updates.
    ///
    /// Account pipes decode and process updates to accounts within the pipeline. This method
    /// requires both an `AccountDecoder` and a `Processor` to handle decoded account data.
    ///
    /// # Parameters
    ///
    /// - `decoder`: An `AccountDecoder` that decodes the account data.
    /// - `processor`: A `Processor` that processes the decoded account data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .account(MyAccountDecoder, MyAccountProcessor);
    /// ```
    ///
    pub fn account<T: Send + Sync + 'static>(
        mut self,
        decoder: impl for<'a> AccountDecoder<'a, AccountType = T> + Send + Sync + 'static,
        processor: impl Processor<InputType = (AccountMetadata, DecodedAccount<T>)>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        self.account_pipes.push(Box::new(AccountPipe {
            decoder: Box::new(decoder),
            processor: Box::new(processor),
        }));
        self
    }

    /// Adds an account deletion pipe to handle account deletion events.
    ///
    /// Account deletion pipes process deletions of accounts, with a `Processor` to handle
    /// the deletion events as they occur.
    ///
    /// # Parameters
    ///
    /// - `processor`: A `Processor` that processes account deletion events.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .account_deletions(MyAccountDeletionProcessor);
    /// ```
    ///
    pub fn account_deletions(
        mut self,
        processor: impl Processor<InputType = AccountDeletion> + Send + Sync + 'static,
    ) -> Self {
        self.account_deletion_pipes
            .push(Box::new(AccountDeletionPipe {
                processor: Box::new(processor),
            }));
        self
    }

    /// Adds an instruction pipe to process instructions within transactions.
    ///
    /// Instruction pipes decode and process individual instructions,
    /// enabling specialized handling of various instruction types.
    ///
    /// # Parameters
    ///
    /// - `decoder`: An `InstructionDecoder` for decoding instructions from transaction data.
    /// - `processor`: A `Processor` that processes decoded instruction data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .instruction(MyDecoder, MyInstructionProcessor);
    /// ```
    ///
    pub fn instruction<T: Send + Sync + 'static>(
        mut self,
        decoder: impl for<'a> InstructionDecoder<'a, InstructionType = T> + Send + Sync + 'static,
        processor: impl Processor<
                InputType = (
                    InstructionMetadata,
                    DecodedInstruction<T>,
                    Vec<NestedInstruction>,
                ),
            > + Send
            + Sync
            + 'static,
    ) -> Self {
        self.instruction_pipes.push(Box::new(InstructionPipe {
            decoder: Box::new(decoder),
            processor: Box::new(processor),
        }));
        self
    }

    /// Adds a transaction pipe for processing full transaction data.
    ///
    /// This method requires a transaction schema for decoding and a `Processor` to handle
    /// the processed transaction data.
    ///
    /// # Parameters
    ///
    /// - `schema`: A `TransactionSchema` used to match and interpret transaction data.
    /// - `processor`: A `Processor` that processes the decoded transaction data.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .transaction(MY_SCHEMA.clone(), MyTransactionProcessor);
    /// ```
    ///
    pub fn transaction<T, U>(
        mut self,
        schema: TransactionSchema<T>,
        processor: impl Processor<InputType = U> + Send + Sync + 'static,
    ) -> Self
    where
        T: InstructionDecoderCollection + 'static,
        U: DeserializeOwned + Send + Sync + 'static,
    {
        self.transaction_pipes
            .push(Box::new(TransactionPipe::<T, U>::new(schema, processor)));
        self
    }

    /// Adds a metrics component to the pipeline for performance tracking.
    ///
    /// This component collects and reports on pipeline metrics, providing insights into
    /// performance and operational statistics.
    ///
    /// # Parameters
    ///
    /// - `metrics`: An instance of a `Metrics` implementation, used to gather and report metrics.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .metrics(Arc::new(LogMetrics::new()));
    /// ```
    ///
    pub fn metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        self.metrics.push(metrics);
        self
    }

    /// Sets the interval for flushing metrics data.
    ///
    /// This value defines the frequency, in seconds, at which metrics data is flushed
    /// from memory. If not set, a default interval is used.
    ///
    /// # Parameters
    ///
    /// - `interval`: The flush interval for metrics, in seconds.
    ///
    /// # Example
    ///
    /// ```rust
    /// let builder = PipelineBuilder::new()
    ///     .metrics_flush_interval(60);
    /// ```
    ///
    pub fn metrics_flush_interval(mut self, interval: u64) -> Self {
        self.metrics_flush_interval = Some(interval);
        self
    }

    /// Builds and returns a `Pipeline` configured with the specified components.
    ///
    /// After configuring the `PipelineBuilder` with data sources, pipes, and metrics,
    /// call this method to create the final `Pipeline` instance ready for operation.
    ///
    /// # Returns
    ///
    /// Returns a `CarbonResult<Pipeline>` containing the configured `Pipeline`, or an error
    /// if any part of the configuration is invalid.
    ///
    /// # Example
    ///
    /// ```rust
    /// carbon_core::pipeline::Pipeline::builder()
    /// .datasource(transaction_crawler)
    /// .metrics(Arc::new(LogMetrics::new()))
    /// .metrics(Arc::new(PrometheusMetrics::new()))
    /// .instruction(
    ///    TestProgramDecoder,
    ///    TestProgramProcessor
    /// )
    /// .account(
    ///     TestProgramDecoder,
    ///     TestProgramAccountProcessor
    /// )
    /// .transaction(TEST_SCHEMA.clone(), TestProgramTransactionProcessor)
    /// .account_deletions(TestProgramAccountDeletionProcessor)
    /// .build()?
    /// ```
    ///
    pub fn build(self) -> CarbonResult<Pipeline> {
        Ok(Pipeline {
            datasources: self.datasources,
            account_pipes: self.account_pipes,
            account_deletion_pipes: self.account_deletion_pipes,
            instruction_pipes: self.instruction_pipes,
            transaction_pipes: self.transaction_pipes,
            metrics: self.metrics,
            metrics_flush_interval: self.metrics_flush_interval,
        })
    }
}
