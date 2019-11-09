# CKB Genesis Block Generator Specification

## Generator Input

The generator accepts following parameters as input:

* RPC ENDPOINT, default to `http://localhost:8114`
* Epoch number E, default to 89
* Embedded issued cells generated from CSV
* Embedded issued cells for miner competition round 1 ~ 4 and round 5 stage 1 and stage 2.

## Expected Behavior

* Create a new chain spec for mainnet
* Promote user to use the following command to setup the configuration files for mainnet.

```
 ckb init --import-spec /path/to/generated/chain-spec.toml --chain mainnet 
```

## How to create chain spec

Following is the template chain spec.

```
name = "ckb"

[genesis]
version = 0
parent_hash = "0x0000000000000000000000000000000000000000000000000000000000000000"
# TODO: set to the timestamp of the last block in testnet v0.24.0
timestamp = 1572674400069
# TODO: computed from the last epochs in testnet v0.24.0
compact_target = 0x1c00e904
uncles_hash = "0x0000000000000000000000000000000000000000000000000000000000000000"
nonce = "0x0"

[genesis.genesis_cell]
# TODO: Replace 0x000...00 with the last block hash in testnet v0.24.0
message = "lina 0x0000000000000000000000000000000000000000000000000000000000000000"

[genesis.genesis_cell.lock]
code_hash = "0x0000000000000000000000000000000000000000000000000000000000000000"
args = "0x"
hash_type = "data"

# An array list paths to system cell files, which is absolute or relative to
# the directory containing this config file.
[[genesis.system_cells]]
file = { bundled = "specs/cells/secp256k1_blake160_sighash_all" }
create_type_id = true
capacity = 100_000_0000_0000
[[genesis.system_cells]]
file = { bundled = "specs/cells/dao" }
create_type_id = true
capacity = 16_000_0000_0000
[[genesis.system_cells]]
file = { bundled = "specs/cells/secp256k1_data" }
create_type_id = false
capacity = 1_048_617_0000_0000
[[genesis.system_cells]]
file = { bundled = "specs/cells/secp256k1_blake160_multisig_all" }
create_type_id = true
capacity = 100_000_0000_0000

[genesis.system_cells_lock]
code_hash = "0x0000000000000000000000000000000000000000000000000000000000000000"
args = "0x"
hash_type = "data"

# Dep group cells
[[genesis.dep_groups]]
name = "secp256k1_blake160_sighash_all"
files = [
  { bundled = "specs/cells/secp256k1_data" },
  { bundled = "specs/cells/secp256k1_blake160_sighash_all" },
]
[[genesis.dep_groups]]
name = "secp256k1_blake160_multisig_all"
files = [
  { bundled = "specs/cells/secp256k1_data" },
  { bundled = "specs/cells/secp256k1_blake160_multisig_all" },
]

# For first 11 block
[genesis.bootstrap_lock]
code_hash = "0x0000000000000000000000000000000000000000000000000000000000000000"
args = "0x"
hash_type = "data"

# Burn
[[genesis.issued_cells]]
capacity = 8_400_000_000_00000000
lock.code_hash = "0x0000000000000000000000000000000000000000000000000000000000000000"
lock.args = "0x62e907b15cbf27d5425399ebf6f0fb50ebb88f18"
lock.hash_type = "data"

# TODO: Other issued cells starts here
# Public Token Sale: 21.5%
# Ecosystem fund: 17%
# Team: 15%
# Private Sale: 14%
# Strategic Founding Partners: 5%
# Foundation Reserve: 2% - genesis message cell - system cells - dep groups
# Testnet Incentives: 0.5%

[params]
# TODO: Set to the length of the last epoch of testnet
genesis_epoch_length = 1000

[pow]
func = "Eaglesong"
```



### Genesis Timestamp

Set to the timestamp of the last block in epoch number E in the testnet via API ENDPOINT.


### Genesis Compact Target

Define `D(e)` as the difficulty of the epoch number e in the testnet. Define `P(e)` as the sum of all the block primary issuances until epoch number e (including e), in CKBytes.

The average difficulty of the last 4 epochs is

```
d = (D(E-3) + D(E-2) + D(E-1) + D(E)) / 4
```

The mainnet genesis difficulty is


```
d * 1.5 * P(E) / 18000000
```

The difficulty then is converted to compact target.

### Genesis Message

It is formatted as “lina 0x...”, where 0x... is the hash of the last block in epoch number E in the testnet.


### Genesis Epoch Length

Genesis epoch length of the mainnet must set to the length of the epoch E of the testnet.


## Issued Cells

The issued cells are the most complex part and deserved its own chapter.

The template only contains one issued cell, the burned cell.

The issued cells are split into partitions, in the listed order:


* Burn
* Cells generated from CSV
* Foundation Reserve
* Testnet Incentives

### Burn

8.4 billion CKBytes are burned.


### CSV Generated


The CSV contains data for following parts:

* Public Token Sale (21.5%)
* Ecosystem fund (17%)
* Team (15%)
* Private Sale (14%)
* Strategic Founding Partners (5%)


The total CKBytes in the CSV is 24.36 billion.

The CSV has following columns:


* address: mainnet address which must be the Short Payload Format with code hash index 0x00, see [rfc#0021](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0021-ckb-address-format/0021-ckb-address-format.md). The address can be used to restore the public key blake160 hash.
* capacity: Amount of tokens, in CKBytes.
* lock: keep empty if there’s no lock requirements, otherwise set to date in the format YYYY-MM-DD. The date is converted to the timestamp at 00:00 in the UTC timezone.


Each line of the CSV is converted into a issued cell.

If the line has no lock, the issued cell must use the default secp256k1 via type as the lock, the arg is the restored public key hash.

If the line has a lock, the issued cell must use the genesis multisign via type as the lock. The arg is


```
multisig script hash | since lock

where | is bytes concatenation
```


Multisig script hash is the blake160 of the following message: 

```
`S | R | M | N | PubkeyHash`
```

Where [S/R/M/N](https://github.com/nervosnetwork/ckb-system-scripts/blob/master/c/secp256k1_blake160_multisig_all.c#L43) are four single byte unsigned integers, where S = 0, R = 0, M = 1, N = 1.

PubkeyHash is the restored public key hash from the address.

The Since lock is encoded the same with the since in transaction input, see details in [rfc#0017](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0017-tx-valid-since/0017-tx-valid-since.md).

```
Se | Sn | Sd | Sf
```

* Se: 3 bytes epoch number in little endian.
* Sn: 2 bytes epoch numerator in little endian.
* Sd: 2 bytes epoch denominator in little endian. 
* Sf: 1 byte since flag


Since lock is generated from the generator input parameter epoch number E, and the date in the CSV.

First compute the default locked time in seconds:

```
L = seconds elapsed
  since 2019-11-16 UTC 6am
  until the date in CSV at 00:00 in UTC
```

Then

```
Se = floor(L / 14400) + 89 - E
Sn = (L % 14400) * 1800 / 14400
Sd = 1800
Sf = 32
```


Specially, if Se is negative, set both Se and Sn to 0.

The issued cells should keep its original sequence in the CSV.


### Foundation Reserve

Foundation Reserve total is 672 millions. But it should also cover the genesis message cell, system script cells and system dep groups. The capacity is 672 millions subtracting the capacities used by these cells.


* Genesis message cell: 112 CKBytes
* System script cells: 1,264,617 CKBytes
* Dep Groups: 234 CKBytes

So the remaining foundation reserve capacity is 670,735,037 CKBytes.

The foundation reserve lock is

```
address: ckb1qyqyz340d4nhgtx2s75mp5wnavrsu7j5fcwqktprrp
lock: 2020-07-01
```

It is converted to script lock using the same logic mentioned in CSV generated issued cells.


### Testnet Incentives

The miner competition round 1 to 4, and stage 1, 2 of round 5 are precomputed from the history data.

The rewards of round 5 stage 3 are computed from the data via API ENDPOINT, which covers blocks in epoch 0 to E.

All the rewards are aggregated by public key hash. One public key hash will only have a single issued cell. All the testnet incentives are sorted by public key hash in the ascending order.

The block mined by invalid lock are rewarded to the foundation testnet incentives lock. If the total issued testnet incentives are less than 168 millions CKBytes, the remaining part is also rewarded to this lock.

The address for the foundation testnet incentives lock is

```
address: ckb1qyqy6mtud5sgctjwgg6gydd0ea05mr339lnslczzrc
```


The testnet incentive for the special lock must put in the end after all other testnet incentives.
