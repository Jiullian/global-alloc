#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// Imports obligatoires
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem;

extern crate alloc;
use alloc::vec::Vec;

// Structure de données des blocs libres
struct BlocLibre {
    // size : représente la taille du bloc libre en octets
    size: usize,
    // next : pointeur optionnel mutable qui va venir pointer vers le prochain bloc libre
    next: Option<*mut BlocLibre>
}

// Concrètement, on peut s'imaginer ça :
// +-----------+      +-----------+      +-----------+
// | BlocLibre | ---> | BlocLibre | ---> | BlocLibre | ---> None
// +-----------+      +-----------+      +-----------+

// Implémentation de l'allocateur
pub struct AllocateurListeLibre{
    // head : représente le début de la liste chaînée des blocs libres (sera vide ou contiendra un pointeur).
    head : UnsafeCell<Option<*mut BlocLibre>>,
}