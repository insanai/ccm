#import "book.typ": book
#show: book.with(title: "CCM: Code, Change, and Confidence", author: "Vikrant Rathore",
  version: sys.inputs.at("version", default: "0.5.0"), logo: "../../../assets/ccm-logo.svg")
