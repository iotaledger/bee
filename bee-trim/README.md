# bee-trim

Provides low-level trimming algorithms for IOTA transactions to reduce bandwidth and storage consumption.

## Contains
* trim_data: a trimming algorithm that trims the data field only (signature message fragment)
* trim_full: a trimming algorithm that trims all transaction fields (slightly slower as 'trim_data' but better compression rate)

## Reasoning

In general IOTA/Bee transactions have a fixed size of 8019 trits (or 2673 trytes or 1782 bytes). But often times transactions are not completely "filled" and could be sent and stored consuming less resources. Using general purpose compresson algorithms don't come at zero cost and can become a burden on weak-ish devices. Also they cannot use knowledge about the structure of IOTA/Bee transactions.

That's why Bee will use a compromise between bandwidth and storage consumption and CPU utilization. Instead of trying to compress all redundant data in a transaction, this crate provides algorithms to simply trim useless data from one or more transaction fields by using knowledge about how transactions are structured.